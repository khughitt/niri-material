use niri_config::utils::MergeWith as _;
use niri_config::{Config, LayerRule};
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::element::Kind;
#[cfg(test)]
use smithay::backend::renderer::element::{Element as _, Id};
use smithay::backend::renderer::gles::GlesTexProgram;
#[cfg(test)]
use smithay::backend::renderer::utils::CommitCounter;
use smithay::desktop::utils::bbox_from_surface_tree;
use smithay::desktop::{LayerSurface, PopupKind, PopupManager};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle, Scale, Size};
use smithay::wayland::compositor::{remove_pre_commit_hook, HookId};
use smithay::wayland::shell::wlr_layer::{ExclusiveZone, Layer};

use super::ResolvedLayerRules;
use crate::animation::Clock;
use crate::layout::shadow::Shadow;
use crate::niri_render_elements;
use crate::render_helpers::background_effect::BackgroundEffectElement;
use crate::render_helpers::damage::ExtraDamage;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::shadow::ShadowRenderElement;
use crate::render_helpers::solid_color::{SolidColorBuffer, SolidColorRenderElement};
use crate::render_helpers::surface::push_elements_from_surface_tree;
use crate::render_helpers::surface_remap::{OutputHue, RemapSetup, RemapUniforms};
use crate::render_helpers::surface_remap_element::SurfaceRemapRenderElement;
use crate::render_helpers::xray::XrayPos;
use crate::render_helpers::{background_effect, RenderCtx};
use crate::utils::{baba_is_float_offset, round_logical_in_physical};

#[derive(Debug)]
pub struct MappedLayer {
    /// The surface itself.
    surface: LayerSurface,

    /// Pre-commit hook that we have on all mapped layer surfaces.
    pre_commit_hook: HookId,

    /// Up-to-date rules.
    rules: ResolvedLayerRules,

    /// Whether to recompute layer rules on the next commit.
    ///
    /// Set in the pre-commit hook when the layer changes; consumed in the commit handler.
    recompute_rules_on_commit: bool,

    /// Buffer to draw instead of the surface when it should be blocked out.
    block_out_buffer: SolidColorBuffer,

    /// The shadow around the surface.
    shadow: Shadow,

    /// The blur config, passed for background effect rendering.
    blur_config: niri_config::Blur,

    /// The view size for the layer surface's output.
    view_size: Size<f64, Logical>,

    /// Scale of the output the layer surface is on (and rounds its sizes to).
    scale: f64,

    /// Clock for driving animations.
    clock: Clock,

    /// The surface remap for this layer's rule, rebuilt on config and rule changes.
    remap: Option<RemapSetup>,

    /// What the layer's output shows this frame.
    shell_hue: OutputHue,

    /// Uniforms last drawn, so a change damages the surface once.
    remap_drawn: Option<RemapUniforms>,

    /// Damage for the surface and for its pop-ups when the remap changes.
    remap_damage: ExtraDamage,
    popup_remap_damage: ExtraDamage,
}

niri_render_elements! {
    LayerSurfaceRenderElement<R> => {
        Wayland = WaylandSurfaceRenderElement<R>,
        SurfaceRemap = SurfaceRemapRenderElement<R>,
        SolidColor = SolidColorRenderElement,
        Shadow = ShadowRenderElement,
        BackgroundEffect = BackgroundEffectElement,
        Damage = ExtraDamage,
    }
}

/// A remapped layer's uniforms with its compiled program; `None` draws the
/// surface as the client sent it.
type RemapProgram = Option<(RemapUniforms, GlesTexProgram)>;

fn remap_setup(rules: &ResolvedLayerRules, config: &Config) -> Option<RemapSetup> {
    let remap = rules.surface_remap.as_ref()?;
    let material = config
        .materials
        .iter()
        .find(|m| m.name == remap.material.name)
        .expect("surface-remap materials are validated at config load");
    Some(RemapSetup::new(remap, &material.resolve().glass))
}

/// What `surface`'s tree covers when drawn at `loc`: its subsurfaces
/// included, as the remap wraps each of them.
fn surface_tree_extent(surface: &WlSurface, loc: Point<f64, Logical>) -> Rectangle<f64, Logical> {
    let mut extent = bbox_from_surface_tree(surface, (0, 0)).to_f64();
    extent.loc += loc;
    extent
}

/// A surface element as pushed: through the remap program when there is one.
fn surface_element<R: NiriRenderer>(
    elem: WaylandSurfaceRenderElement<R>,
    program: Option<&(RemapUniforms, GlesTexProgram)>,
) -> LayerSurfaceRenderElement<R> {
    match program {
        Some((uniforms, program)) => {
            SurfaceRemapRenderElement::new(elem, program.clone(), *uniforms).into()
        }
        None => elem.into(),
    }
}

impl MappedLayer {
    pub fn new(
        surface: LayerSurface,
        pre_commit_hook: HookId,
        rules: ResolvedLayerRules,
        view_size: Size<f64, Logical>,
        scale: f64,
        clock: Clock,
        config: &Config,
    ) -> Self {
        let mut shadow_config = config.layout.shadow;
        // Shadows for layer surfaces need to be explicitly enabled.
        shadow_config.on = false;
        shadow_config.merge_with(&rules.shadow);

        let remap = remap_setup(&rules, config);

        Self {
            surface,
            pre_commit_hook,
            rules,
            recompute_rules_on_commit: false,
            block_out_buffer: SolidColorBuffer::new((0., 0.), [0., 0., 0., 1.]),
            view_size,
            scale,
            shadow: Shadow::new(shadow_config),
            blur_config: config.blur,
            clock,
            remap,
            shell_hue: OutputHue::default(),
            remap_drawn: None,
            remap_damage: ExtraDamage::new(),
            popup_remap_damage: ExtraDamage::new(),
        }
    }

    pub fn update_config(&mut self, config: &Config) {
        let mut shadow_config = config.layout.shadow;
        // Shadows for layer surfaces need to be explicitly enabled.
        shadow_config.on = false;
        shadow_config.merge_with(&self.rules.shadow);
        self.shadow.update_config(shadow_config);

        self.blur_config = config.blur;
        self.remap = remap_setup(&self.rules, config);
    }

    pub fn update_shaders(&mut self) {
        self.shadow.update_shaders();
    }

    pub fn update_sizes(&mut self, view_size: Size<f64, Logical>, scale: f64) {
        self.view_size = view_size;
        self.scale = scale;
    }

    pub fn update_render_elements(&mut self, size: Size<f64, Logical>) {
        // Round to physical pixels.
        let size = size
            .to_physical_precise_round(self.scale)
            .to_logical(self.scale);

        self.block_out_buffer.resize(size);

        let radius = self.rules.geometry_corner_radius.unwrap_or_default();
        // FIXME: is_active based on keyboard focus?
        self.shadow
            .update_render_elements(size, true, radius, self.scale, 1.);
    }

    pub fn are_animations_ongoing(&self) -> bool {
        self.rules.baba_is_float || (self.remap.is_some() && self.shell_hue.animating)
    }

    pub fn remap_uniforms(&self) -> Option<RemapUniforms> {
        self.remap
            .as_ref()
            .map(|setup| setup.uniforms(self.shell_hue.blend))
    }

    #[cfg(test)]
    pub fn shell_hue(&self) -> OutputHue {
        self.shell_hue
    }

    /// Takes this frame's hue for the layer's output; a change in what the
    /// remap draws damages the surface and its pop-ups once.
    pub fn set_shell_hue(&mut self, hue: OutputHue) {
        self.shell_hue = hue;
        let uniforms = self.remap_uniforms();
        if uniforms != self.remap_drawn {
            self.remap_drawn = uniforms;
            self.remap_damage.damage_all();
            self.popup_remap_damage.damage_all();
        }
    }

    #[cfg(test)]
    pub fn remap_damage_commit(&self) -> CommitCounter {
        self.remap_damage.current_commit()
    }

    #[cfg(test)]
    pub fn popup_remap_damage_commit(&self) -> CommitCounter {
        self.popup_remap_damage.current_commit()
    }

    #[cfg(test)]
    pub fn remap_damage_id(&self) -> &Id {
        self.remap_damage.id()
    }

    #[cfg(test)]
    pub fn popup_remap_damage_id(&self) -> &Id {
        self.popup_remap_damage.id()
    }

    /// The remap's uniforms and program, when the layer has a remap and the
    /// program compiled. A program that failed to compile has logged
    /// `error compiling surface remap shader`; the layer then draws
    /// unremapped, as niri does for every custom texture program.
    fn remap_program<R: NiriRenderer>(&self, renderer: &mut R) -> RemapProgram {
        let uniforms = self.remap_uniforms()?;
        let program = SurfaceRemapRenderElement::shader(renderer)?.clone();
        Some((uniforms, program))
    }

    pub fn surface(&self) -> &LayerSurface {
        &self.surface
    }

    pub fn rules(&self) -> &ResolvedLayerRules {
        &self.rules
    }

    /// Recomputes the resolved layer rules and returns whether they changed.
    pub fn recompute_layer_rules(&mut self, rules: &[LayerRule], is_at_startup: bool) -> bool {
        let new_rules = ResolvedLayerRules::compute(rules, &self.surface, is_at_startup);
        if new_rules == self.rules {
            return false;
        }

        self.rules = new_rules;
        true
    }

    pub fn set_recompute_rules_on_commit(&mut self) {
        self.recompute_rules_on_commit = true;
    }

    pub fn take_recompute_rules_on_commit(&mut self) -> bool {
        std::mem::take(&mut self.recompute_rules_on_commit)
    }

    pub fn place_within_backdrop(&self) -> bool {
        if !self.rules.place_within_backdrop {
            return false;
        }

        if self.surface.layer() != Layer::Background {
            return false;
        }

        let state = self.surface.cached_state();
        if state.exclusive_zone != ExclusiveZone::DontCare {
            return false;
        }

        true
    }

    pub fn bob_offset(&self) -> Point<f64, Logical> {
        if !self.rules.baba_is_float {
            return Point::from((0., 0.));
        }

        let y = baba_is_float_offset(self.clock.now(), self.view_size.h);
        let y = round_logical_in_physical(self.scale, y);
        Point::from((0., y))
    }

    pub fn render_normal<R: NiriRenderer>(
        &self,
        mut ctx: RenderCtx<R>,
        ns: Option<usize>,
        location: Point<f64, Logical>,
        xray_pos: XrayPos,
        push: &mut dyn FnMut(LayerSurfaceRenderElement<R>),
    ) {
        let scale = Scale::from(self.scale);
        let alpha = self.rules.opacity.unwrap_or(1.).clamp(0., 1.);

        let bob_offset = self.bob_offset();
        let location = location + bob_offset;
        let xray_pos = xray_pos.offset(bob_offset);

        let surface = self.surface.wl_surface();

        let should_block_out = ctx.target.should_block_out(self.rules.block_out_from);
        if should_block_out {
            // Round to physical pixels.
            let location = location.to_physical_precise_round(scale).to_logical(scale);

            // FIXME: take geometry-corner-radius into account.
            let elem = SolidColorRenderElement::from_buffer(
                &self.block_out_buffer,
                location,
                alpha,
                Kind::Unspecified,
            );
            push(elem.into());
        } else {
            // Layer surfaces don't have extra geometry like windows.
            let buf_pos = location;

            if self.remap.is_some() {
                // Damage before the pixels it redraws: elements are front to
                // back, and damage behind a declared opaque region is skipped.
                let extent = surface_tree_extent(surface, buf_pos);
                push(self.remap_damage.render(extent).into());
            }
            let program = self.remap_program(ctx.renderer);
            push_elements_from_surface_tree(
                ctx.renderer,
                surface,
                buf_pos.to_physical_precise_round(scale),
                scale,
                alpha,
                Kind::ScanoutCandidate,
                &mut |elem| push(surface_element(elem, program.as_ref())),
            );
        }

        let location = location.to_physical_precise_round(scale).to_logical(scale);
        self.shadow
            .render(ctx.renderer, location, &mut |elem| push(elem.into()));

        let geometry = Rectangle::new(location, self.block_out_buffer.size());
        let surface_off = Point::new(0., 0.); // No geometry on layer surfaces.
        let surface_anim_scale = Scale::from(1.);
        let radius = self.rules.geometry_corner_radius.unwrap_or_default();
        background_effect::render_for_tile(
            ctx.as_gles(),
            ns,
            geometry,
            self.scale,
            false,
            surface,
            surface_off,
            surface_anim_scale,
            self.blur_config,
            radius,
            self.rules.background_effect,
            should_block_out,
            xray_pos,
            &mut |elem| push(elem.into()),
        );
    }

    pub fn render_popups<R: NiriRenderer>(
        &self,
        mut ctx: RenderCtx<R>,
        ns: Option<usize>,
        location: Point<f64, Logical>,
        xray_pos: XrayPos,
        push: &mut dyn FnMut(LayerSurfaceRenderElement<R>),
    ) {
        if ctx.target.should_block_out(self.rules.block_out_from) {
            return;
        }

        let scale = Scale::from(self.scale);
        let alpha = self.rules.opacity.unwrap_or(1.).clamp(0., 1.);

        let bob_offset = self.bob_offset();
        let location = location + bob_offset;
        let xray_pos = xray_pos.offset(bob_offset);

        let surface = self.surface.wl_surface();
        let program = self.remap_program(ctx.renderer);
        if self.remap.is_some() {
            // One rectangle over the xdg popups the remap reaches, pushed
            // before them for the same front-to-back reason as the layer's.
            let bounds = PopupManager::popups_for_surface(surface)
                .filter(|(popup, _)| PopupClass::of(popup).follows_layer())
                .map(|(popup, offset)| {
                    let surface_loc = location + (offset - popup.geometry().loc).to_f64();
                    surface_tree_extent(popup.wl_surface(), surface_loc)
                })
                .reduce(|a, b| a.merge(b));
            if let Some(bounds) = bounds {
                push(self.popup_remap_damage.render(bounds).into());
            }
        }

        for (popup, offset) in PopupManager::popups_for_surface(surface) {
            let (popup_rules, popup_program) =
                popup_treatment(PopupClass::of(&popup), self.rules.popups, program.clone());
            let alpha = alpha * popup_rules.opacity.unwrap_or(1.).clamp(0., 1.);

            let surface = popup.wl_surface();
            let popup_geo = popup.geometry();
            let surface_loc = location + (offset - popup_geo.loc).to_f64();

            push_elements_from_surface_tree(
                ctx.renderer,
                surface,
                surface_loc.to_physical_precise_round(scale),
                scale,
                alpha,
                Kind::ScanoutCandidate,
                &mut |elem| push(surface_element(elem, popup_program.as_ref())),
            );

            let geometry = Rectangle::new(location + offset.to_f64(), popup_geo.size.to_f64());
            let surface_off = popup_geo.loc.upscale(-1).to_f64();
            let surface_anim_scale = Scale::from(1.);
            let mut effect = popup_rules.background_effect;
            // Default xray to false for pop-ups since they're always on top of something.
            if effect.xray.is_none() {
                effect.xray = Some(false);
            }
            let xray_pos = xray_pos.offset(offset.to_f64());
            background_effect::render_for_tile(
                ctx.as_gles(),
                ns,
                geometry,
                self.scale,
                false,
                surface,
                surface_off,
                surface_anim_scale,
                self.blur_config,
                popup_rules.geometry_corner_radius.unwrap_or_default(),
                effect,
                false,
                xray_pos,
                &mut |elem| push(elem.into()),
            );
        }
    }
}

/// Which kind of popup a layer surface's popup is, for rule decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupClass {
    Xdg,
    InputMethod,
}

impl PopupClass {
    fn of(popup: &PopupKind) -> Self {
        match popup {
            PopupKind::Xdg(_) => Self::Xdg,
            PopupKind::InputMethod(_) => Self::InputMethod,
        }
    }

    /// Whether the popup is the layer's own, and so takes its popup rules
    /// and remap. Input-method popups (candidate windows) belong to the
    /// input method's client.
    fn follows_layer(self) -> bool {
        match self {
            Self::Xdg => true,
            Self::InputMethod => false,
        }
    }
}

/// The popup rules and remap a popup gets: the layer's when the popup
/// follows the layer, neither otherwise.
fn popup_treatment<T>(
    class: PopupClass,
    popups: niri_config::ResolvedPopupsRules,
    remap: Option<T>,
) -> (niri_config::ResolvedPopupsRules, Option<T>) {
    if class.follows_layer() {
        (popups, remap)
    } else {
        (niri_config::ResolvedPopupsRules::default(), None)
    }
}

impl Drop for MappedLayer {
    fn drop(&mut self) {
        remove_pre_commit_hook(self.surface.wl_surface(), &self.pre_commit_hook);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_method_popups_take_neither_rules_nor_remap() {
        let popups = niri_config::ResolvedPopupsRules {
            opacity: Some(0.5),
            ..Default::default()
        };
        assert_eq!(
            popup_treatment(PopupClass::InputMethod, popups, Some(7u8)),
            (niri_config::ResolvedPopupsRules::default(), None)
        );
        assert_eq!(
            popup_treatment(PopupClass::Xdg, popups, Some(7u8)),
            (popups, Some(7u8))
        );
    }
}
