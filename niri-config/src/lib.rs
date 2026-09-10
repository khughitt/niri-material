//! niri config parsing.
//!
//! The config can be constructed from multiple files (includes). To support this, many types are
//! split into two. For example, `Layout` and `LayoutPart` where `Layout` is the final config and
//! `LayoutPart` is one part parsed from one config file.
//!
//! The convention for `Default` impls is to set the initial values before the parsing occurs.
//! Then, parsing will update the values with those parsed from the config.
//!
//! The `Default` values match those from `default-config.kdl` in almost all cases, with a notable
//! exception of `binds {}` and some window rules.

#[macro_use]
extern crate tracing;

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use knuffel::errors::DecodeError;
use knuffel::Decode as _;
use miette::{miette, Context as _, IntoDiagnostic as _};

#[macro_use]
pub mod macros;

pub mod animations;
pub mod appearance;
pub mod binds;
pub mod debug;
pub mod error;
pub mod gestures;
pub mod input;
pub mod layer_rule;
pub mod layout;
pub mod material;
pub mod misc;
pub mod output;
pub mod recent_windows;
pub mod signal;
pub mod utils;
pub mod window_rule;
pub mod workspace;

pub use crate::animations::{Animation, Animations};
pub use crate::appearance::*;
pub use crate::binds::*;
pub use crate::debug::Debug;
pub use crate::error::{ConfigIncludeError, ConfigParseResult};
pub use crate::gestures::Gestures;
pub use crate::input::{Input, ModKey, ScrollMethod, TrackLayout, WarpMouseToFocusMode, Xkb};
pub use crate::layer_rule::LayerRule;
pub use crate::layout::*;
pub use crate::material::optics::iridescence::{Iridescence, ResolvedIridescence};
pub use crate::material::optics::noise::{Noise, NoiseType, ResolvedNoise};
pub use crate::material::optics::saturation::ResolvedSaturation;
pub use crate::material::{
    AccentResponse, AttentionResponse, FocusResponse, Glass, ImpulseResponse, Material,
    MaterialRef, Positive, ResolvedGlass, ResolvedMaterial, ResolvedResponse, Response,
};
pub use crate::misc::*;
pub use crate::output::{Output, OutputName, Outputs, Position, Vrr};
use crate::recent_windows::RecentWindowsPart;
pub use crate::recent_windows::{MruDirection, MruFilter, MruPreviews, MruScope, RecentWindows};
pub use crate::signal::{Signal, SignalMotionPolicy, SignalPart};
pub use crate::utils::FloatOrInt;
use crate::utils::{Flag, MergeWith as _};
pub use crate::window_rule::{
    FloatingPosition, PopupsRule, RelativeTo, ResolvedPopupsRules, WindowRule,
};
pub use crate::workspace::{Workspace, WorkspaceLayoutPart};

const RECURSION_LIMIT: u8 = 10;

#[derive(Debug, Default, PartialEq)]
pub struct Config {
    pub input: Input,
    pub outputs: Outputs,
    pub spawn_at_startup: Vec<SpawnAtStartup>,
    pub spawn_sh_at_startup: Vec<SpawnShAtStartup>,
    pub layout: Layout,
    pub prefer_no_csd: bool,
    pub cursor: Cursor,
    pub screenshot_path: ScreenshotPath,
    pub clipboard: Clipboard,
    pub hotkey_overlay: HotkeyOverlay,
    pub config_notification: ConfigNotification,
    pub animations: Animations,
    pub blur: Blur,
    pub signal: Signal,
    pub gestures: Gestures,
    pub overview: Overview,
    pub environment: Environment,
    pub xwayland_satellite: XwaylandSatellite,
    pub window_rules: Vec<WindowRule>,
    pub materials: Vec<Material>,
    pub layer_rules: Vec<LayerRule>,
    pub binds: Binds,
    pub switch_events: SwitchBinds,
    pub debug: Debug,
    pub workspaces: Vec<Workspace>,
    pub recent_windows: RecentWindows,
}

#[derive(Debug, Clone)]
pub enum ConfigPath {
    /// Explicitly set config path.
    ///
    /// Load the config only from this path, never create it.
    Explicit(PathBuf),

    /// Default config path.
    ///
    /// Prioritize the user path, fallback to the system path, fallback to creating the user path
    /// at compositor startup.
    Regular {
        /// User config path, usually `$XDG_CONFIG_HOME/niri/config.kdl`.
        user_path: PathBuf,
        /// System config path, usually `/etc/niri/config.kdl`.
        system_path: PathBuf,
    },
}

// Newtypes for putting information into the knuffel context.
struct BasePath(PathBuf);
struct RootBase(PathBuf);
struct Recursion(u8);
// Name of the file currently being decoded, for error messages that cannot
// carry a span across a file boundary.
struct FileName(String);
#[derive(Default)]
struct Includes(Vec<PathBuf>);
#[derive(Default)]
struct IncludeErrors(Vec<knuffel::Error>);
// Used for recursive include detection.
//
// We don't *need* it because we have a recursion limit, but it makes for nicer error messages.
struct IncludeStack(HashSet<PathBuf>);
struct SawMruBinds(Rc<Cell<bool>>);

// Rather than listing all fields and deriving knuffel::Decode, we implement
// knuffel::DecodeChildren by hand, since we need custom logic for every field anyway: we want to
// merge the values into the config from the context as we go to support the positionality of
// includes. The reason we need this type at all is because knuffel's only entry point that allows
// setting default values on a context is `parse_with_context()` that needs a type to parse.
pub struct ConfigPart;

impl<S> knuffel::DecodeChildren<S> for ConfigPart
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_children(
        nodes: &[knuffel::ast::SpannedNode<S>],
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let _span = tracy_client::span!("decode config file");

        let config = ctx.get::<Rc<RefCell<Config>>>().unwrap().clone();
        let includes = ctx.get::<Rc<RefCell<Includes>>>().unwrap().clone();
        let include_errors = ctx.get::<Rc<RefCell<IncludeErrors>>>().unwrap().clone();
        let material_refs = ctx
            .get::<Rc<RefCell<crate::material::MaterialRefs<S>>>>()
            .expect("material refs must be set in the parse context")
            .clone();
        let recursion = ctx.get::<Recursion>().unwrap().0;
        let saw_mru_binds = ctx.get::<SawMruBinds>().unwrap().0.clone();

        let mut seen = HashSet::new();

        for node in nodes {
            let name = &**node.node_name;

            // Within one config file, splitting sections into multiple parts is not allowed to
            // reduce confusion. The exceptions here aren't multipart; they all add new values.
            if !matches!(
                name,
                "output"
                    | "spawn-at-startup"
                    | "spawn-sh-at-startup"
                    | "window-rule"
                    | "layer-rule"
                    | "workspace"
                    | "material"
                    | "include"
            ) && !seen.insert(name)
            {
                ctx.emit_error(DecodeError::unexpected(
                    &node.node_name,
                    "node",
                    format!("duplicate node `{name}`, single node expected"),
                ));
                continue;
            }

            macro_rules! m_merge {
                ($field:ident) => {{
                    let part = knuffel::Decode::decode_node(node, ctx)?;
                    config.borrow_mut().$field.merge_with(&part);
                }};
            }

            macro_rules! m_push {
                ($field:ident) => {{
                    let part = knuffel::Decode::decode_node(node, ctx)?;
                    config.borrow_mut().$field.push(part);
                }};
            }

            match name {
                "input" => m_merge!(input),
                "cursor" => m_merge!(cursor),
                "clipboard" => m_merge!(clipboard),
                "hotkey-overlay" => m_merge!(hotkey_overlay),
                "config-notification" => m_merge!(config_notification),
                "animations" => m_merge!(animations),
                "blur" => m_merge!(blur),
                "signal" => m_merge!(signal),
                "gestures" => m_merge!(gestures),
                "overview" => m_merge!(overview),
                "xwayland-satellite" => m_merge!(xwayland_satellite),
                "switch-events" => m_merge!(switch_events),
                "debug" => m_merge!(debug),

                // Multipart sections.
                "output" => {
                    let part = Output::decode_node(node, ctx)?;
                    config.borrow_mut().outputs.0.push(part);
                }
                "spawn-at-startup" => m_push!(spawn_at_startup),
                "spawn-sh-at-startup" => m_push!(spawn_sh_at_startup),
                "window-rule" => m_push!(window_rules),
                "material" => {
                    let part = Material::decode_node(node, ctx)?;

                    // `resolve` is infallible and runs after the config is
                    // already accepted, so a cross-parameter rule has to be
                    // checked here, where a decode error can still be
                    // emitted. Unlike `validate_material_refs` this needs no
                    // post-include deferral: the rule is per-definition.
                    if let Err(message) = part.validate() {
                        match node.arguments.first() {
                            Some(arg) => ctx.emit_error(DecodeError::unexpected(
                                &arg.literal,
                                "material",
                                message,
                            )),
                            None => {
                                ctx.emit_error(DecodeError::unexpected(node, "material", message))
                            }
                        }
                    }

                    // `config` is shared across includes, unlike a value set
                    // into this file's Context, so this also catches a
                    // duplicate defined in another file.
                    let mut config = config.borrow_mut();
                    if config.materials.iter().any(|m| m.name == part.name) {
                        let message = format!("duplicate material: {}", part.name);
                        // Point at the name argument so the caret lands on
                        // the name rather than the whole block.
                        match node.arguments.first() {
                            Some(arg) => ctx.emit_error(DecodeError::unexpected(
                                &arg.literal,
                                "material",
                                message,
                            )),
                            None => {
                                ctx.emit_error(DecodeError::unexpected(node, "material", message))
                            }
                        }
                    } else {
                        config.materials.push(part);
                    }
                }
                "layer-rule" => m_push!(layer_rules),
                "workspace" => m_push!(workspaces),

                // Single-part sections.
                "binds" => {
                    let part = Binds::decode_node(node, ctx)?;

                    // We replace conflicting binds, rather than error, to support the use-case
                    // where you import some preconfigured-dots.kdl, then override some binds with
                    // your own.
                    let mut config = config.borrow_mut();
                    let binds = &mut config.binds.0;
                    // Remove existing binds matching any new bind.
                    binds.retain(|bind| !part.0.iter().any(|new| new.key == bind.key));
                    // Add all new binds.
                    binds.extend(part.0);
                }
                "environment" => {
                    let part = Environment::decode_node(node, ctx)?;
                    config.borrow_mut().environment.0.extend(part.0);
                }

                "prefer-no-csd" => {
                    config.borrow_mut().prefer_no_csd = Flag::decode_node(node, ctx)?.0
                }

                "screenshot-path" => {
                    let part = knuffel::Decode::decode_node(node, ctx)?;
                    config.borrow_mut().screenshot_path = part;
                }

                "layout" => {
                    let mut part = LayoutPart::decode_node(node, ctx)?;

                    // Preserve the behavior we'd always had for the border section:
                    // - `layout {}` gives border = off
                    // - `layout { border {} }` gives border = on
                    // - `layout { border { off } }` gives border = off
                    //
                    // This behavior is inconsistent with the rest of the config where adding an
                    // empty section generally doesn't change the outcome. Particularly, shadows
                    // are also disabled by default (like borders), and they always had an `on`
                    // instead of an `off` for this reason, so that writing `layout { shadow {} }`
                    // still results in shadow = off, as it should.
                    //
                    // Unfortunately, the default config has always had wording that heavily
                    // implies that `layout { border {} }` enables the borders. This wording is
                    // sure to be present in a lot of users' configs by now, which we can't change.
                    //
                    // Another way to make things consistent would be to default borders to on.
                    // However, that is annoying because it would mean changing many tests that
                    // rely on borders being off by default. This would also contradict the
                    // intended default borders value (off).
                    //
                    // So, let's just work around the problem here, preserving the original
                    // behavior.
                    if recursion == 0 {
                        if let Some(border) = part.border.as_mut() {
                            if !border.on && !border.off {
                                border.on = true;
                            }
                        }
                    }

                    config.borrow_mut().layout.merge_with(&part);
                }

                "recent-windows" => {
                    let part = RecentWindowsPart::decode_node(node, ctx)?;

                    let mut config = config.borrow_mut();

                    // When an MRU binds section is encountered for the first time, clear out the
                    // default MRU binds.
                    if !saw_mru_binds.get() && part.binds.is_some() {
                        saw_mru_binds.set(true);
                        config.recent_windows.binds.clear();
                    }

                    config.recent_windows.merge_with(&part);
                }

                "include" => {
                    // Parse the path argument
                    let mut iter_args = node.arguments.iter();
                    let path_val = iter_args.next().ok_or_else(|| {
                        DecodeError::missing(
                            node,
                            "additional argument for include path is required",
                        )
                    })?;
                    let path: PathBuf = knuffel::traits::DecodeScalar::decode(path_val, ctx)?;

                    // Check for extra arguments
                    if let Some(val) = iter_args.next() {
                        ctx.emit_error(DecodeError::unexpected(
                            &val.literal,
                            "argument",
                            "unexpected argument",
                        ));
                    }

                    // Parse the optional property
                    let mut optional = false;
                    for (name, val) in &node.properties {
                        match &***name {
                            "optional" => {
                                optional = knuffel::traits::DecodeScalar::decode(val, ctx)?;
                            }
                            name_str => {
                                ctx.emit_error(DecodeError::unexpected(
                                    name,
                                    "property",
                                    format!("unexpected property `{}`", name_str.escape_default()),
                                ));
                            }
                        }
                    }

                    // Check for unexpected children
                    for child in node.children() {
                        ctx.emit_error(DecodeError::unexpected(
                            child,
                            "node",
                            format!("unexpected node `{}`", child.node_name.escape_default()),
                        ));
                    }

                    // We use DecodeError::Missing throughout this block because it results in the
                    // least confusing error messages while still allowing to provide a span.

                    // Expand ~ into the home dir
                    let path = if let Ok(rest) = path.strip_prefix("~") {
                        let Some(home) = std::env::home_dir() else {
                            ctx.emit_error(DecodeError::missing(
                                node,
                                format!("error retrieving home directory to expand {path:?}"),
                            ));
                            continue;
                        };

                        home.join(rest)
                    } else {
                        // Otherwise, use the current include base dir
                        let base = ctx.get::<BasePath>().unwrap();
                        base.0.join(path)
                    };

                    let recursion = ctx.get::<Recursion>().unwrap().0 + 1;
                    if recursion == RECURSION_LIMIT {
                        ctx.emit_error(DecodeError::missing(
                            node,
                            format!(
                                "reached the recursion limit; \
                                 includes cannot be {RECURSION_LIMIT} levels deep"
                            ),
                        ));
                        continue;
                    }

                    let Some(filename) = path.file_name().and_then(OsStr::to_str) else {
                        ctx.emit_error(DecodeError::missing(
                            node,
                            "include path doesn't have a valid file name",
                        ));
                        continue;
                    };
                    let base = path.parent().map(Path::to_path_buf).unwrap_or_default();

                    // Check for recursive include for a nicer error message.
                    let mut include_stack = ctx.get::<IncludeStack>().unwrap().0.clone();
                    if !include_stack.insert(path.to_path_buf()) {
                        ctx.emit_error(DecodeError::missing(
                            node,
                            "recursive include (file includes itself)",
                        ));
                        continue;
                    }

                    // Store even if the include fails to read or parse, so it gets watched.
                    includes.borrow_mut().0.push(path.to_path_buf());

                    match fs::read_to_string(&path) {
                        Ok(text) => {
                            // Try to get filename relative to the root base config folder for
                            // clearer error messages.
                            let root_base = &ctx.get::<RootBase>().unwrap().0;
                            // Failing to strip prefix usually means absolute path; show it in full.
                            let relative_path = path.strip_prefix(root_base).ok().unwrap_or(&path);
                            let filename = relative_path.to_str().unwrap_or(filename);

                            let part = knuffel::parse_with_context::<
                                ConfigPart,
                                knuffel::span::Span,
                                _,
                            >(filename, &text, |ctx| {
                                ctx.set(BasePath(base));
                                ctx.set(RootBase(root_base.clone()));
                                ctx.set(Recursion(recursion));
                                ctx.set(FileName(filename.to_string()));
                                ctx.set(includes.clone());
                                ctx.set(include_errors.clone());
                                ctx.set(material_refs.clone());
                                ctx.set(IncludeStack(include_stack));
                                ctx.set(SawMruBinds(saw_mru_binds.clone()));
                                ctx.set(config.clone());
                            });

                            match part {
                                Ok(_) => {}
                                Err(err) => {
                                    include_errors.borrow_mut().0.push(err);

                                    ctx.emit_error(DecodeError::missing(
                                        node,
                                        "failed to parse included config",
                                    ));
                                }
                            }
                        }
                        Err(err) => {
                            if optional && err.kind() == std::io::ErrorKind::NotFound {
                                // Warn about missing optional includes
                                warn!("optional include not found: {path:?}");
                            } else {
                                // Report all other errors normally
                                ctx.emit_error(DecodeError::missing(
                                    node,
                                    format!("failed to read included config from {path:?}: {err}"),
                                ));
                            }
                        }
                    }
                }

                name => {
                    ctx.emit_error(DecodeError::unexpected(
                        node,
                        "node",
                        format!("unexpected node `{}`", name.escape_default()),
                    ));
                }
            }
        }

        // Materials may be defined after the rules that reference them, or in
        // an include, so this check waits until everything has merged.
        if recursion == 0 {
            let config = config.borrow();
            crate::material::validate_material_refs(
                &material_refs.borrow(),
                &config.materials,
                ctx,
            );
        }

        Ok(Self)
    }
}

impl Config {
    pub fn load_default() -> Self {
        let res = Config::parse(
            Path::new("default-config.kdl"),
            include_str!("../../resources/default-config.kdl"),
        );

        // Includes in the default config can break its parsing at runtime.
        assert!(
            res.includes.is_empty(),
            "default config must not have includes",
        );

        res.config.unwrap()
    }

    pub fn load(path: &Path) -> ConfigParseResult<Self, miette::Report> {
        let contents = match fs::read_to_string(path) {
            Ok(x) => x,
            Err(err) => {
                return ConfigParseResult::from_err(
                    miette!(err).context(format!("error reading {path:?}")),
                );
            }
        };

        Self::parse(path, &contents).map_config_res(|res| {
            let config = res.context("error parsing")?;
            debug!("loaded config from {path:?}");
            Ok(config)
        })
    }

    pub fn parse(path: &Path, text: &str) -> ConfigParseResult<Self, ConfigIncludeError> {
        let base = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let filename = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("config.kdl");

        let config = Rc::new(RefCell::new(Config::default()));
        let includes = Rc::new(RefCell::new(Includes(Vec::new())));
        let include_errors = Rc::new(RefCell::new(IncludeErrors(Vec::new())));
        let material_refs = Rc::new(RefCell::new(crate::material::MaterialRefs::<
            knuffel::span::Span,
        >::default()));
        let include_stack = HashSet::from([path.to_path_buf()]);

        let part = knuffel::parse_with_context::<ConfigPart, knuffel::span::Span, _>(
            filename,
            text,
            |ctx| {
                ctx.set(BasePath(base.clone()));
                ctx.set(RootBase(base));
                ctx.set(Recursion(0));
                ctx.set(FileName(filename.to_string()));
                ctx.set(includes.clone());
                ctx.set(include_errors.clone());
                ctx.set(material_refs.clone());
                ctx.set(IncludeStack(include_stack));
                ctx.set(SawMruBinds(Rc::new(Cell::new(false))));
                ctx.set(config.clone());
            },
        );

        let includes = includes.take().0;
        let include_errors = include_errors.take().0;
        let config = part
            .map(|_| config.take())
            .map_err(move |err| ConfigIncludeError {
                main: err,
                includes: include_errors,
            });

        ConfigParseResult { config, includes }
    }

    pub fn parse_mem(text: &str) -> Result<Self, ConfigIncludeError> {
        Self::parse(Path::new("config.kdl"), text).config
    }
}

impl ConfigPath {
    /// Loads the config, returns an error if it doesn't exist.
    pub fn load(&self) -> ConfigParseResult<Config, miette::Report> {
        let _span = tracy_client::span!("ConfigPath::load");

        self.load_inner(|user_path, system_path| {
            Err(miette!(
                "no config file found; create one at {user_path:?} or {system_path:?}",
            ))
        })
        .map_config_res(|res| res.context("error loading config"))
    }

    /// Loads the config, or creates it if it doesn't exist.
    ///
    /// Returns a tuple containing the path that was created, if any, and the loaded config.
    ///
    /// If the config was created, but for some reason could not be read afterwards,
    /// this may return `(Some(_), Err(_))`.
    pub fn load_or_create(&self) -> (Option<&Path>, ConfigParseResult<Config, miette::Report>) {
        let _span = tracy_client::span!("ConfigPath::load_or_create");

        let mut created_at = None;

        let result = self
            .load_inner(|user_path, _| {
                Self::create(user_path, &mut created_at)
                    .map(|()| user_path)
                    .with_context(|| format!("error creating config at {user_path:?}"))
            })
            .map_config_res(|res| res.context("error loading config"));

        (created_at, result)
    }

    fn load_inner<'a>(
        &'a self,
        maybe_create: impl FnOnce(&'a Path, &'a Path) -> miette::Result<&'a Path>,
    ) -> ConfigParseResult<Config, miette::Report> {
        let path = match self {
            ConfigPath::Explicit(path) => path.as_path(),
            ConfigPath::Regular {
                user_path,
                system_path,
            } => {
                if user_path.exists() {
                    user_path.as_path()
                } else if system_path.exists() {
                    system_path.as_path()
                } else {
                    match maybe_create(user_path.as_path(), system_path.as_path()) {
                        Ok(x) => x,
                        Err(err) => return ConfigParseResult::from_err(miette!(err)),
                    }
                }
            }
        };
        Config::load(path)
    }

    fn create<'a>(path: &'a Path, created_at: &mut Option<&'a Path>) -> miette::Result<()> {
        if let Some(default_parent) = path.parent() {
            fs::create_dir_all(default_parent)
                .into_diagnostic()
                .with_context(|| format!("error creating config directory {default_parent:?}"))?;
        }

        // Create the config and fill it with the default config if it doesn't exist.
        let mut new_file = match File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
        {
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => return Ok(()),
            res => res,
        }
        .into_diagnostic()
        .with_context(|| format!("error opening config file at {path:?}"))?;

        *created_at = Some(path);

        let default = include_bytes!("../../resources/default-config.kdl");

        new_file
            .write_all(default)
            .into_diagnostic()
            .with_context(|| format!("error writing default config to {path:?}"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::{assert_debug_snapshot, assert_snapshot};
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn can_create_default_config() {
        let _ = Config::load_default();
    }

    #[test]
    fn default_repeat_params() {
        let config = Config::parse_mem("").unwrap();
        assert_eq!(config.input.keyboard.repeat_delay, 600);
        assert_eq!(config.input.keyboard.repeat_rate, 25);
    }

    #[track_caller]
    fn do_parse(text: &str) -> Config {
        Config::parse_mem(text)
            .map_err(miette::Report::new)
            .unwrap()
    }

    #[track_caller]
    fn do_parse_err(text: &str) -> String {
        let err = Config::parse_mem(text).expect_err("config should have failed to parse");
        format!("{:?}", miette::Report::new(err))
    }

    /// Writes `files` into a fresh directory and parses the first one, so
    /// `include` resolution runs against real paths. `parse_mem` cannot do
    /// this: it has no directory to resolve includes against.
    #[track_caller]
    fn parse_files(files: &[(&str, &str)]) -> Result<Config, ConfigIncludeError> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);

        let dir = std::env::temp_dir().join(format!(
            "niri-config-material-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();

        for (name, text) in files {
            fs::write(dir.join(name), text).unwrap();
        }

        let res = Config::parse(&dir.join(files[0].0), files[0].1).config;
        let _ = fs::remove_dir_all(&dir);
        res
    }

    #[track_caller]
    fn parse_files_err(files: &[(&str, &str)]) -> String {
        let err = parse_files(files).expect_err("config should have failed to parse");
        format!("{:?}", miette::Report::new(err))
    }

    #[test]
    fn signal_block_and_matches_parse() {
        let parsed = parse_files(&[(
            "config.kdl",
            r##"
            signal {
                motion "reduced"
            }

            animations {
                material-signal { duration-ms 250; curve "ease-out-quad"; }
            }

            window-rule {
                match signal-source="^familiar$" signal-tag="^cats/"
                opacity 0.9
            }
            "##,
        )])
        .unwrap();

        assert_eq!(parsed.signal.motion, crate::SignalMotionPolicy::Reduced);
        assert_eq!(
            parsed.animations.material_signal.0.kind,
            crate::animations::Kind::Easing(crate::animations::EasingParams {
                duration_ms: 250,
                curve: crate::animations::Curve::EaseOutQuad,
            })
        );
        let m = &parsed.window_rules[0].matches[0];
        assert!(m.signal_source.as_ref().unwrap().0.is_match("familiar"));
        assert!(m.signal_tag.as_ref().unwrap().0.is_match("cats/ginger"));
    }

    #[test]
    fn signal_motion_defaults_to_full() {
        let parsed = parse_files(&[("config.kdl", "")]).unwrap();
        assert_eq!(parsed.signal.motion, crate::SignalMotionPolicy::Full);
        assert_eq!(
            parsed.animations.material_signal.0.kind,
            crate::animations::Kind::Easing(crate::animations::EasingParams {
                duration_ms: 400,
                curve: crate::animations::Curve::EaseOutCubic,
            })
        );
    }

    #[test]
    fn signal_motion_rejects_unknown_value() {
        let err = parse_files_err(&[("config.kdl", "signal { motion \"loud\"\n}")]);
        assert!(err.contains("unknown"), "{err}");
    }

    #[test]
    fn response_blocks_resolve_with_inheritance() {
        let parsed = parse_files(&[(
            "config.kdl",
            r##"
            material "tg" {
                glass { bevel 12; offset-x 0; offset-y 0; }
                response "default" {
                    accent "ring"
                    attention "rim-orbit"
                    ping "ripple"
                    done "sweep"
                    error "flash"
                    ring-inset 6
                    ring-width 2
                }
                response "loud" {
                    attention "ring-pulse"
                }
            }
            window-rule {
                match app-id="^kitty$"
                material "tg" response="loud"
            }
            "##,
        )])
        .unwrap();

        let m = parsed.materials[0].resolve();
        let loud = m.response(Some("loud"));
        assert_eq!(loud.attention, crate::AttentionResponse::RingPulse);
        assert_eq!(loud.accent, crate::AccentResponse::Ring);
        assert_eq!(loud.done, crate::ImpulseResponse::Sweep);
        assert_eq!(loud.ring_inset, 6.);
        let r = parsed.window_rules[0].material.as_ref().unwrap();
        assert_eq!(r.name, "tg");
        assert_eq!(r.response.as_deref(), Some("loud"));
    }

    #[test]
    fn material_without_response_block_gets_builtin_default() {
        let parsed = parse_files(&[("config.kdl", r#"material "tg" { glass {}; }"#)]).unwrap();
        let d = parsed.materials[0].resolve().response(None);
        assert_eq!(d.accent, crate::AccentResponse::Ring);
        assert_eq!(d.attention, crate::AttentionResponse::RimOrbit);
        assert_eq!(d.ping, crate::ImpulseResponse::Ripple);
        assert_eq!(d.done, crate::ImpulseResponse::Sweep);
        assert_eq!(d.error, crate::ImpulseResponse::Flash);
        assert_eq!((d.ring_inset, d.ring_width), (5., 2.6));
    }

    #[test]
    fn focus_response_fields_parse_and_default() {
        let parsed = parse_files(&[(
            "config.kdl",
            r##"
            material "tg" {
                glass { light-ior 3; }
                response "default" {
                    focus "none"
                    ring-color "#ff8800"
                    ring-drift-hz 20
                }
                response "still" { ring-drift-hz 0; }
            }
            "##,
        )])
        .unwrap();
        let m = parsed.materials[0].resolve();
        assert_eq!(m.glass.light_ior, 3.);
        let d = m.response(None);
        assert_eq!(d.focus, crate::FocusResponse::None);
        assert_eq!(
            d.ring_color,
            Color::from_rgba8_unpremul(0xff, 0x88, 0x00, 0xff)
        );
        assert_eq!(d.ring_drift_hz, 20.);
        let still = m.response(Some("still"));
        assert_eq!(
            still.focus,
            crate::FocusResponse::None,
            "inherited from default"
        );
        assert_eq!(still.ring_drift_hz, 0.);

        let builtin = parse_files(&[("config.kdl", r#"material "tg" { glass {}; }"#)])
            .unwrap()
            .materials[0]
            .resolve();
        let b = builtin.response(None);
        assert_eq!(b.focus, crate::FocusResponse::RingLight);
        assert_eq!(
            b.ring_color,
            Color::from_rgba8_unpremul(0xcc, 0xcc, 0xff, 0xff)
        );
        assert_eq!(b.ring_drift_hz, 15.);
        assert_eq!((b.ring_inset, b.ring_width), (5., 2.6));
        assert_eq!(builtin.glass.light_ior, 6.);
    }

    #[test]
    fn ring_width_must_be_positive_but_may_be_fractional() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" { ring-width 0; }; }"#,
        )]);
        assert!(err.contains("ring-width must be positive"), "{err}");
        let ok = parse_files(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" { ring-width 0.5; }; }"#,
        )])
        .unwrap();
        assert_eq!(ok.materials[0].resolve().response(None).ring_width, 0.5);
    }

    #[test]
    fn ring_drift_hz_is_zero_or_at_least_one() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" { ring-drift-hz 0.5; }; }"#,
        )]);
        assert!(
            err.contains("ring-drift-hz must be 0 or at least 1"),
            "{err}"
        );
        for ok in ["0", "1", "7.5", "30"] {
            let parsed = parse_files(&[(
                "config.kdl",
                &format!(r#"material "tg" {{ glass {{}}; response "default" {{ ring-drift-hz {ok}; }}; }}"#),
            )])
            .unwrap();
            assert_eq!(
                parsed.materials[0].resolve().response(None).ring_drift_hz,
                ok.parse::<f64>().unwrap()
            );
        }
    }

    #[test]
    fn focus_response_rejects_out_of_range() {
        for body in [
            r#"material "tg" { glass {}; response "default" { ring-drift-hz 31; }; }"#,
            r#"material "tg" { glass { light-ior 13; }; }"#,
            r#"material "tg" { glass { light-ior 0.5; }; }"#,
            r#"material "tg" { glass {}; response "default" { focus "glow"; }; }"#,
        ] {
            parse_files_err(&[("config.kdl", body)]);
        }
    }

    #[test]
    fn response_block_without_default_is_an_error() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "loud" { attention "ring-pulse"; }; }"#,
        )]);
        assert!(err.contains("missing response \"default\""), "{err}");
    }

    #[test]
    fn duplicate_response_name_is_an_error() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" {}; response "default" {}; }"#,
        )]);
        assert!(err.contains("duplicate response: default"), "{err}");
    }

    #[test]
    fn ring_must_fit_in_bevel() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass { bevel 4; }; response "default" { ring-inset 3; ring-width 2; }; }"#,
        )]);
        assert!(
            err.contains("ring-inset + ring-width must not exceed bevel"),
            "{err}"
        );
    }

    #[test]
    fn unknown_response_reference_is_an_error() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"
            material "tg" { glass {}; }
            window-rule { material "tg" response="loud"; }
            "#,
        )]);
        assert!(err.contains("unknown response: loud"), "{err}");
    }

    #[test]
    fn material_reference_rejects_duplicate_response_property() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"
            material "tg" { glass {}; }
            window-rule { material "tg" response="default" response="default"; }
            "#,
        )]);
        assert!(err.contains("duplicate property `response`"), "{err}");
    }

    #[test]
    fn material_reference_rejects_malformed_nodes() {
        for (bad, needle) in [
            (r#"material "tg" "extra""#, "unexpected argument"),
            (r#"material "tg" { glass {}; }"#, "unexpected node"),
            (r#"(typed)material "tg""#, "no type name expected"),
            (r#"material (typed)"tg""#, "type name"),
            (r#"material "tg" response=(typed)"default""#, "type name"),
            (r#"material "tg" bogus="x""#, "unexpected property"),
        ] {
            let err = parse_files_err(&[(
                "config.kdl",
                &format!("material \"tg\" {{ glass {{}}; }}\nwindow-rule {{ {bad}; }}"),
            )]);
            assert!(err.contains(needle), "{bad}: {err}");
        }
    }

    #[test]
    fn impulse_selector_follows_policy() {
        use crate::{ImpulseResponse, ResolvedResponse, SignalMotionPolicy};
        let r = ResolvedResponse::default();
        assert_eq!(
            r.impulse_selector(niri_ipc::ImpulseKind::Error, SignalMotionPolicy::Full),
            Some(ImpulseResponse::Flash as u8)
        );
        assert_eq!(
            r.impulse_selector(niri_ipc::ImpulseKind::Error, SignalMotionPolicy::Reduced),
            Some(ImpulseResponse::Sweep as u8)
        );
        assert_eq!(
            r.impulse_selector(niri_ipc::ImpulseKind::Error, SignalMotionPolicy::Off),
            None
        );
        let mut none = r;
        none.done = ImpulseResponse::None;
        assert_eq!(
            none.impulse_selector(niri_ipc::ImpulseKind::Done, SignalMotionPolicy::Full),
            None
        );
    }

    #[test]
    fn window_rule_material_reference_resolves() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {}
            }

            window-rule {
                match app-id="Alacritty"
                material "frost"
            }
            "##,
        );
        assert_eq!(
            parsed.window_rules[0]
                .material
                .as_ref()
                .map(|m| m.name.as_str()),
            Some("frost")
        );
    }

    #[test]
    fn window_rule_material_reference_may_precede_definition() {
        // Definition order must not matter.
        let parsed = do_parse(
            r##"
            window-rule {
                material "frost"
            }

            material "frost" {
                glass {}
            }
            "##,
        );
        assert_eq!(parsed.materials.len(), 1);
        assert_eq!(
            parsed.window_rules[0]
                .material
                .as_ref()
                .map(|m| m.name.as_str()),
            Some("frost")
        );
    }

    #[test]
    fn window_rule_rejects_unknown_material() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {}
            }

            window-rule {
                material "smoke"
            }
            "##,
        );
        assert!(err.contains("unknown material: smoke"), "{err}");
    }

    #[test]
    fn material_duplicate_across_include_is_rejected() {
        // Task 2's arm exists for exactly this: a context-local name set
        // would not see across the include boundary.
        let err = parse_files_err(&[
            (
                "main.kdl",
                "include \"other.kdl\"\nmaterial \"frost\" { glass {}; }\n",
            ),
            ("other.kdl", "material \"frost\" { glass {}; }\n"),
        ]);
        assert!(err.contains("duplicate material: frost"), "{err}");
    }

    #[test]
    fn material_reference_in_include_resolves_to_later_root_definition() {
        // The reference is inside the include; the definition comes after
        // the include line in the root. Only a fully deferred check accepts
        // this.
        let parsed = parse_files(&[
            (
                "main.kdl",
                "include \"other.kdl\"\nmaterial \"frost\" { glass {}; }\n",
            ),
            ("other.kdl", "window-rule { material \"frost\"; }\n"),
        ])
        .unwrap();
        assert_eq!(parsed.materials.len(), 1);
        assert_eq!(parsed.window_rules.len(), 1);
    }

    #[test]
    fn unknown_material_in_include_names_its_file() {
        // An included reference has no usable span in the root context, so
        // the message must identify the file itself.
        let err = parse_files_err(&[
            ("main.kdl", "include \"other.kdl\"\n"),
            ("other.kdl", "window-rule { material \"smoke\"; }\n"),
        ]);
        assert!(err.contains("unknown material: smoke"), "{err}");
        assert!(err.contains("other.kdl"), "{err}");
    }

    #[test]
    fn material_definition_in_include_satisfies_root_reference() {
        let parsed = parse_files(&[
            (
                "main.kdl",
                "window-rule { material \"frost\"; }\ninclude \"other.kdl\"\n",
            ),
            ("other.kdl", "material \"frost\" { glass {}; }\n"),
        ])
        .unwrap();
        assert_eq!(parsed.materials.len(), 1);
    }

    #[test]
    fn material_parses_full_glass_block() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {
                    ior 1.4
                    thickness 32
                    attenuation-color "#ff0000"
                    attenuation-distance 80
                    chromatic-aberration 0.25
                    distortion 0.5 scale=1.5
                    anisotropic-blur 0.75
                    roughness 0.08
                    jelly-flex 0.01
                    jelly-ripple 0.2
                    bevel 20
                    offset-x -8
                    offset-y 4
                }
            }
            "##,
        );

        assert_eq!(parsed.materials.len(), 1);
        assert_eq!(parsed.materials[0].name, "frost");
        assert_eq!(
            parsed.materials[0].resolve().glass,
            ResolvedGlass {
                ior: 1.4,
                thickness: 32.,
                attenuation_color: Color::from_rgba8_unpremul(255, 0, 0, 255),
                attenuation_distance: 80.,
                chromatic_aberration: 0.25,
                distortion: 0.5,
                distortion_scale: 1.5,
                anisotropic_blur: 0.75,
                roughness: 0.08,
                iridescence: ResolvedIridescence::default(),
                backdrop_blur: false,
                jelly_flex: 0.01,
                jelly_ripple: 0.2,
                bevel: 20.,
                offset_x: -8.,
                offset_y: 4.,
                noise: ResolvedNoise::default(),
                saturation: ResolvedSaturation::default(),
                light_ior: 6.,
            }
        );
    }

    #[test]
    fn material_omitted_parameters_take_design_defaults() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {}
            }
            "##,
        );

        // Every omitted parameter falls back to the v1 design §4 default.
        assert_eq!(
            parsed.materials[0].resolve().glass,
            ResolvedGlass::default()
        );
    }

    #[test]
    fn backdrop_blur_defaults_to_off() {
        assert!(!ResolvedGlass::default().backdrop_blur);
    }

    #[test]
    fn roughness_defaults_to_zero() {
        assert_eq!(ResolvedGlass::default().roughness, 0.);
    }

    #[test]
    fn roughness_rejects_values_outside_zero_and_one() {
        for value in ["-0.01", "1.01"] {
            let err = do_parse_err(&format!(
                "material \"frost\" {{ glass {{ roughness {value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 0 and 1"), "{err}");
        }
    }

    #[test]
    fn backdrop_blur_parses_and_resolves() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {
                    backdrop-blur true
                }
            }
            "##,
        );
        assert!(parsed.materials[0].resolve().glass.backdrop_blur);
    }

    #[test]
    fn glass_noise_and_saturation_parse_as_written() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {
                    noise 0.02
                    saturation 0.85
                }
            }
            "##,
        );
        let glass = parsed.materials[0].resolve().glass;
        assert_eq!(glass.noise.amount, Some(0.02));
        assert_eq!(glass.saturation.amount, Some(0.85));
    }

    #[test]
    fn glass_noise_and_saturation_resolve_independently() {
        let noise_only = do_parse(r##"material "frost" { glass { noise 0.5; }; }"##);
        let glass = noise_only.materials[0].resolve().glass;
        assert_eq!(glass.noise.amount, Some(0.5));
        assert_eq!(glass.saturation.amount, None);

        let saturation_only = do_parse(r##"material "frost" { glass { saturation 0; }; }"##);
        let glass = saturation_only.materials[0].resolve().glass;
        assert_eq!(glass.noise.amount, None);
        assert_eq!(glass.saturation.amount, Some(0.));
    }

    #[test]
    fn saturation_resolves_through_its_optic() {
        let written = do_parse(r##"material "frost" { glass { saturation 0.85; }; }"##);
        assert_eq!(
            written.materials[0].resolve().glass.saturation,
            ResolvedSaturation { amount: Some(0.85) }
        );
        let omitted = do_parse(r##"material "frost" { glass {}; }"##);
        assert_eq!(
            omitted.materials[0].resolve().glass.saturation,
            ResolvedSaturation::default()
        );
        assert_eq!(ResolvedSaturation::default().amount, None);
    }

    #[test]
    fn noise_resolves_through_its_optic() {
        let written = do_parse(r##"material "frost" { glass { noise 0.5 type="fine"; }; }"##);
        assert_eq!(
            written.materials[0].resolve().glass.noise,
            ResolvedNoise {
                amount: Some(0.5),
                kind: NoiseType::Fine,
            }
        );
        let omitted = do_parse(r##"material "frost" { glass {}; }"##);
        assert_eq!(
            omitted.materials[0].resolve().glass.noise,
            ResolvedNoise::default()
        );
        assert_eq!(ResolvedNoise::default().kind, NoiseType::White);
        assert_eq!(ResolvedNoise::default().amount, None);
    }

    #[test]
    fn iridescence_resolves_through_its_optic() {
        let written = do_parse(r##"material "gem" { glass { iridescence 0.8; }; }"##);
        assert_eq!(
            written.materials[0].resolve().glass.iridescence,
            ResolvedIridescence { amount: 0.8 }
        );
        let omitted = do_parse(r##"material "gem" { glass {}; }"##);
        assert_eq!(
            omitted.materials[0].resolve().glass.iridescence,
            ResolvedIridescence::default()
        );
        assert_eq!(ResolvedIridescence::default().amount, 0.);
    }

    #[test]
    fn glass_iridescence_rejects_values_outside_zero_and_one() {
        for value in ["-0.01", "1.01"] {
            let err = do_parse_err(&format!(
                "material \"gem\" {{ glass {{ iridescence {value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 0 and 1"), "{err}");
        }
    }

    #[test]
    fn glass_noise_and_saturation_default_to_inherit() {
        let d = ResolvedGlass::default();
        assert_eq!(d.noise.amount, None);
        assert_eq!(d.saturation.amount, None);
    }

    #[test]
    fn glass_noise_rejects_values_outside_zero_and_one() {
        for value in ["-0.01", "1.01"] {
            let err = do_parse_err(&format!(
                "material \"frost\" {{ glass {{ noise {value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 0 and 1"), "{err}");
        }
    }

    #[test]
    fn glass_saturation_rejects_values_outside_zero_and_three() {
        for value in ["-0.01", "3.01"] {
            let err = do_parse_err(&format!(
                "material \"frost\" {{ glass {{ saturation {value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 0 and 3"), "{err}");
        }
    }

    #[test]
    fn glass_noise_type_parses_each_value() {
        for (written, expected) in [
            ("white", NoiseType::White),
            ("fine", NoiseType::Fine),
            ("lightness", NoiseType::Lightness),
        ] {
            let parsed = do_parse(&format!(
                "material \"frost\" {{ glass {{ noise 0.02 type=\"{written}\"; }}; }}\n"
            ));
            let glass = parsed.materials[0].resolve().glass;
            assert_eq!(glass.noise.amount, Some(0.02), "{written}");
            assert_eq!(glass.noise.kind, expected, "{written}");
        }
    }

    #[test]
    fn glass_noise_type_rejects_an_unknown_value() {
        let err = do_parse_err("material \"frost\" { glass { noise 0.02 type=\"blue\"; }; }\n");
        assert!(err.contains("unknown NoiseType value: blue"), "{err}");
    }

    #[test]
    fn glass_noise_type_cannot_be_written_without_an_amount() {
        let err = do_parse_err("material \"frost\" { glass { noise type=\"fine\"; }; }\n");
        assert!(!err.is_empty());
    }

    #[test]
    fn an_omitted_noise_type_resolves_to_white_and_keeps_the_amount_rule() {
        let written = do_parse(r##"material "frost" { glass { noise 0.5; }; }"##);
        let glass = written.materials[0].resolve().glass;
        assert_eq!(glass.noise.amount, Some(0.5));
        assert_eq!(glass.noise.kind, NoiseType::White);

        let omitted = do_parse(r##"material "frost" { glass {}; }"##);
        let glass = omitted.materials[0].resolve().glass;
        assert_eq!(glass.noise.amount, None);
        assert_eq!(glass.noise.kind, NoiseType::White);
        assert_eq!(ResolvedGlass::default().noise.kind, NoiseType::White);
    }

    #[test]
    fn an_omitted_backdrop_blur_resolves_to_the_default() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {}
            }
            "##,
        );
        assert!(!parsed.materials[0].resolve().glass.backdrop_blur);
    }

    #[test]
    fn backdrop_blur_rejects_a_non_boolean() {
        do_parse_err(
            r##"
            material "frost" {
                glass {
                    backdrop-blur 0.5
                }
            }
            "##,
        );
    }

    #[test]
    fn material_rejects_out_of_range_parameter() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    ior 4.0
                }
            }
            "##,
        );
        assert!(err.contains("value must be between 1 and 3"), "{err}");
    }

    #[test]
    fn material_accepts_bevel_and_offset() {
        let config = do_parse(
            r##"
            material "frost" {
                glass {
                    bevel 20
                    offset-x -8
                    offset-y 4
                }
            }
            "##,
        );
        let g = config.materials[0].resolve().glass;
        assert_eq!(g.bevel, 20.);
        assert_eq!(g.offset_x, -8.);
        assert_eq!(g.offset_y, 4.);
    }

    #[test]
    fn material_defaults_bevel_and_offset() {
        let config = do_parse(
            r##"
            material "frost" {
                glass {}
            }
            "##,
        );
        let g = config.materials[0].resolve().glass;
        assert_eq!(g.bevel, 12.);
        assert_eq!(g.offset_x, 6.);
        assert_eq!(g.offset_y, 6.);
    }

    #[test]
    fn material_rejects_removed_lip_and_shift() {
        for node in ["lip 6", "shift-x 6", "shift-y 6"] {
            let err = do_parse_err(&format!(
                "material \"frost\" {{\n    glass {{\n        {node}\n    }}\n}}\n"
            ));
            assert!(err.contains("unexpected node"), "{node}: {err}");
        }
    }

    #[test]
    fn material_rejects_out_of_range_bevel() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    bevel 200
                }
            }
            "##,
        );
        assert!(err.contains("value must be between 0 and 128"), "{err}");
    }

    #[test]
    fn material_rejects_offset_exceeding_bevel() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    bevel 10
                    offset-y -12
                }
            }
            "##,
        );
        assert!(err.contains("offset must not exceed bevel"), "{err}");
    }

    #[test]
    fn material_offset_rule_applies_inside_an_include() {
        // Includes decode through the same `"material"` arm, so the check
        // must fire there rather than only on the root file.
        let err = parse_files_err(&[
            ("main.kdl", "include \"other.kdl\"\n"),
            (
                "other.kdl",
                "material \"frost\" { glass { bevel 4; offset-x 9; }; }\n",
            ),
        ]);
        assert!(err.contains("offset must not exceed bevel"), "{err}");
    }

    #[test]
    fn material_rejects_out_of_range_fractional_parameter() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    jelly-flex 0.5
                }
            }
            "##,
        );
        assert!(err.contains("value must be between 0 and 0.02"), "{err}");
    }

    #[test]
    fn material_rejects_zero_attenuation_distance() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    attenuation-distance 0
                }
            }
            "##,
        );
        assert!(
            err.contains("value must be greater than 0 and at most 65535"),
            "{err}"
        );
    }

    #[test]
    fn material_rejects_removed_samples() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    samples 8
                }
            }
            "##,
        );
        assert!(err.contains("unexpected node"), "{err}");
    }

    #[test]
    fn material_accepts_distortion_with_scale() {
        let config = do_parse(
            r##"
            material "frost" {
                glass {
                    distortion 0.5 scale=1.5
                }
            }
            "##,
        );
        let g = config.materials[0].resolve().glass;
        assert_eq!(g.distortion, 0.5);
        assert_eq!(g.distortion_scale, 1.5);
    }

    #[test]
    fn material_distortion_scale_defaults_without_the_property() {
        let config = do_parse(
            r##"
            material "frost" {
                glass {
                    distortion 0.5
                }
            }
            "##,
        );
        let g = config.materials[0].resolve().glass;
        assert_eq!(g.distortion_scale, 0.5);
    }

    #[test]
    fn material_rejects_removed_distortion_scale_node() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    distortion-scale 1.5
                }
            }
            "##,
        );
        assert!(err.contains("unexpected node"), "{err}");
    }

    #[test]
    fn material_rejects_a_scale_without_an_amplitude() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {
                    distortion scale=1.5
                }
            }
            "##,
        );
        // The amplitude is a required argument, so an orphan scale is a
        // parse error rather than an accepted no-op.
        assert!(!err.is_empty(), "{err}");
    }

    #[test]
    fn material_rejects_duplicate_name() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {}
            }

            material "frost" {
                glass {
                    ior 1.2
                }
            }
            "##,
        );
        assert!(err.contains("duplicate material: frost"), "{err}");
    }

    #[test]
    fn material_allows_distinct_names() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {}
            }

            material "clear" {
                glass {}
            }
            "##,
        );
        assert_eq!(parsed.materials.len(), 2);
        assert_eq!(parsed.materials[1].name, "clear");
    }

    #[test]
    fn material_rejects_missing_type_block() {
        let err = do_parse_err(r##"material "frost" {}"##);
        assert!(err.contains("glass"), "{err}");
    }

    #[test]
    fn material_rejects_two_type_blocks() {
        let err = do_parse_err(
            r##"
            material "frost" {
                glass {}
                glass {}
            }
            "##,
        );
        assert!(
            err.contains("duplicate node `glass`, single node expected"),
            "{err}"
        );
    }

    #[test]
    fn parse() {
        let parsed = do_parse(
            r##"
            input {
                keyboard {
                    repeat-delay 600
                    repeat-rate 25
                    track-layout "window"
                    xkb {
                        layout "us,ru"
                        options "grp:win_space_toggle"
                    }
                }

                touchpad {
                    tap
                    dwt
                    dwtp
                    drag true
                    click-method "clickfinger"
                    accel-speed 0.2
                    accel-profile "flat"
                    scroll-method "two-finger"
                    scroll-button 272
                    scroll-button-lock
                    tap-button-map "left-middle-right"
                    disabled-on-external-mouse
                    scroll-factor 0.9
                }

                mouse {
                    natural-scroll
                    accel-speed 0.4
                    accel-profile "flat"
                    scroll-method "no-scroll"
                    scroll-button 273
                    middle-emulation
                    scroll-factor 0.2
                }

                trackpoint {
                    off
                    natural-scroll
                    accel-speed 0.0
                    accel-profile "flat"
                    scroll-method "on-button-down"
                    scroll-button 274
                }

                trackball {
                    off
                    natural-scroll
                    accel-speed 0.0
                    accel-profile "flat"
                    scroll-method "edge"
                    scroll-button 275
                    scroll-button-lock
                    left-handed
                    middle-emulation
                }

                tablet {
                    map-to-output "eDP-1"
                    map-to-focused-output
                    calibration-matrix 1.0 2.0 3.0 \
                                       4.0 5.0 6.0
                }

                touch {
                    map-to-output "eDP-1"
                }

                disable-power-key-handling

                warp-mouse-to-focus
                focus-follows-mouse
                workspace-auto-back-and-forth

                mod-key "Mod5"
                mod-key-nested "Super"
            }

            output "eDP-1" {
                focus-at-startup
                scale 2
                transform "flipped-90"
                position x=10 y=20
                mode "1920x1080@144"
                variable-refresh-rate on-demand=true
                background-color "rgba(25, 25, 102, 1.0)"
                hot-corners {
                    off
                    top-left
                    top-right
                    bottom-left
                    bottom-right
                }
            }

            output "eDP-2" {
                mode custom=true "1920x1080@144"
            }

            output "eDP-3" {
                modeline 173.00  1920 2048 2248 2576  1080 1083 1088 1120 "-hsync" "+vsync"
            }

            layout {
                focus-ring {
                    width 5
                    active-color 0 100 200 255
                    inactive-color 255 200 100 0
                    active-gradient from="rgba(10, 20, 30, 1.0)" to="#0080ffff" relative-to="workspace-view"
                }

                border {
                    width 3
                    inactive-color "rgba(255, 200, 100, 0.0)"
                }

                shadow {
                    offset x=10 y=-20
                }

                tab-indicator {
                    width 10
                    position "top"
                }

                preset-column-widths {
                    proportion 0.25
                    proportion 0.5
                    fixed 960
                    fixed 1280
                }

                preset-window-heights {
                    proportion 0.25
                    proportion 0.5
                    fixed 960
                    fixed 1280
                }

                default-column-width { proportion 0.25; }

                gaps 8

                struts {
                    left 1
                    right 2
                    top 3
                }

                center-focused-column "on-overflow"

                default-column-display "tabbed"

                insert-hint {
                    color "rgb(255, 200, 127)"
                    gradient from="rgba(10, 20, 30, 1.0)" to="#0080ffff" relative-to="workspace-view"
                }
            }

            spawn-at-startup "alacritty" "-e" "fish"
            spawn-sh-at-startup "qs -c ~/source/qs/MyAwesomeShell"

            prefer-no-csd

            cursor {
                xcursor-theme "breeze_cursors"
                xcursor-size 16
                hide-when-typing
                hide-after-inactive-ms 3000
            }

            screenshot-path "~/Screenshots/screenshot.png"

            clipboard {
                disable-primary
            }

            hotkey-overlay {
                skip-at-startup
            }

            animations {
                slowdown 2.0

                workspace-switch {
                    spring damping-ratio=1.0 stiffness=1000 epsilon=0.0001
                }

                horizontal-view-movement {
                    duration-ms 100
                    curve "ease-out-expo"
                }

                window-open { off; }

                window-close {
                    curve "cubic-bezier" 0.05 0.7 0.1 1  
                }

                recent-windows-close {
                    off
                }
            }

            gestures {
                dnd-edge-view-scroll {
                    trigger-width 10
                    max-speed 50
                }
            }

            environment {
                QT_QPA_PLATFORM "wayland"
                DISPLAY null
            }

            window-rule {
                match app-id=".*alacritty"
                exclude title="~"
                exclude is-active=true is-focused=false

                open-on-output "eDP-1"
                open-maximized true
                open-fullscreen false
                open-floating false
                open-focused true
                default-window-height { fixed 500; }
                default-column-display "tabbed"
                default-floating-position x=100 y=-200 relative-to="bottom-left"

                focus-ring {
                    off
                    width 3
                }

                border {
                    on
                    width 8.5
                }

                tab-indicator {
                    active-color "#f00"
                }
            }

            layer-rule {
                match namespace="^notifications$"
                block-out-from "screencast"
            }

            binds {
                Mod+Escape hotkey-overlay-title="Inhibit" { toggle-keyboard-shortcuts-inhibit; }
                Mod+Shift+Escape allow-inhibiting=true { toggle-keyboard-shortcuts-inhibit; }
                Mod+T allow-when-locked=true { spawn "alacritty"; }
                Mod+Q hotkey-overlay-title=null { close-window; }
                Mod+Shift+H { focus-monitor-left; }
                Mod+Shift+O { focus-monitor "eDP-1"; }
                Mod+Ctrl+Shift+L { move-window-to-monitor-right; }
                Mod+Ctrl+Alt+O { move-window-to-monitor "eDP-1"; }
                Mod+Ctrl+Alt+P { move-column-to-monitor "DP-1"; }
                Mod+Comma { consume-window-into-column; }
                Mod+1 { focus-workspace 1; }
                Mod+Shift+1 { focus-workspace "workspace-1"; }
                Mod+Shift+E allow-inhibiting=false { quit skip-confirmation=true; }
                Mod+WheelScrollDown cooldown-ms=150 { focus-workspace-down; }
                Super+Alt+S allow-when-locked=true { spawn-sh "pkill orca || exec orca"; }
            }

            switch-events {
                tablet-mode-on { spawn "bash" "-c" "gsettings set org.gnome.desktop.a11y.applications screen-keyboard-enabled true"; }
                tablet-mode-off { spawn "bash" "-c" "gsettings set org.gnome.desktop.a11y.applications screen-keyboard-enabled false"; }
            }

            debug {
                render-drm-device "/dev/dri/renderD129"
                ignore-drm-device "/dev/dri/renderD128"
                ignore-drm-device "/dev/dri/renderD130"
            }

            workspace "workspace-1" {
                open-on-output "eDP-1"
            }
            workspace "workspace-2"
            workspace "workspace-3"

            recent-windows {
                off

                highlight {
                    padding 15
                    active-color "#00ff00"
                }

                previews {
                    max-height 960
                }

                binds {
                    Alt+Tab { next-window; }
                    Alt+grave { next-window filter="app-id"; }
                    Super+Tab { next-window scope="output"; }
                }
            }
            "##,
        );

        assert_debug_snapshot!(parsed, @r#"
        Config {
            input: Input {
                keyboard: Keyboard {
                    xkb: Xkb {
                        rules: "",
                        model: "",
                        layout: "us,ru",
                        variant: "",
                        options: Some(
                            "grp:win_space_toggle",
                        ),
                        file: None,
                    },
                    repeat_delay: 600,
                    repeat_rate: 25,
                    track_layout: Window,
                    numlock: false,
                },
                touchpad: Touchpad {
                    off: false,
                    tap: true,
                    dwt: true,
                    dwtp: true,
                    drag: Some(
                        true,
                    ),
                    drag_lock: false,
                    natural_scroll: false,
                    click_method: Some(
                        Clickfinger,
                    ),
                    accel_speed: FloatOrInt(
                        0.2,
                    ),
                    accel_profile: Some(
                        Flat,
                    ),
                    scroll_method: Some(
                        TwoFinger,
                    ),
                    scroll_button: Some(
                        272,
                    ),
                    scroll_button_lock: true,
                    tap_button_map: Some(
                        LeftMiddleRight,
                    ),
                    left_handed: false,
                    disabled_on_external_mouse: true,
                    middle_emulation: false,
                    scroll_factor: Some(
                        ScrollFactor {
                            base: Some(
                                FloatOrInt(
                                    0.9,
                                ),
                            ),
                            horizontal: None,
                            vertical: None,
                        },
                    ),
                },
                mouse: Mouse {
                    off: false,
                    natural_scroll: true,
                    accel_speed: FloatOrInt(
                        0.4,
                    ),
                    accel_profile: Some(
                        Flat,
                    ),
                    scroll_method: Some(
                        NoScroll,
                    ),
                    scroll_button: Some(
                        273,
                    ),
                    scroll_button_lock: false,
                    left_handed: false,
                    middle_emulation: true,
                    scroll_factor: Some(
                        ScrollFactor {
                            base: Some(
                                FloatOrInt(
                                    0.2,
                                ),
                            ),
                            horizontal: None,
                            vertical: None,
                        },
                    ),
                },
                trackpoint: Trackpoint {
                    off: true,
                    natural_scroll: true,
                    accel_speed: FloatOrInt(
                        0.0,
                    ),
                    accel_profile: Some(
                        Flat,
                    ),
                    scroll_method: Some(
                        OnButtonDown,
                    ),
                    scroll_button: Some(
                        274,
                    ),
                    scroll_button_lock: false,
                    left_handed: false,
                    middle_emulation: false,
                },
                trackball: Trackball {
                    off: true,
                    natural_scroll: true,
                    accel_speed: FloatOrInt(
                        0.0,
                    ),
                    accel_profile: Some(
                        Flat,
                    ),
                    scroll_method: Some(
                        Edge,
                    ),
                    scroll_button: Some(
                        275,
                    ),
                    scroll_button_lock: true,
                    left_handed: true,
                    middle_emulation: true,
                },
                tablet: Tablet {
                    off: false,
                    calibration_matrix: Some(
                        [
                            1.0,
                            2.0,
                            3.0,
                            4.0,
                            5.0,
                            6.0,
                        ],
                    ),
                    map_to_output: Some(
                        "eDP-1",
                    ),
                    map_to_focused_output: true,
                    left_handed: false,
                },
                touch: Touch {
                    off: false,
                    calibration_matrix: None,
                    map_to_output: Some(
                        "eDP-1",
                    ),
                },
                disable_power_key_handling: true,
                warp_mouse_to_focus: Some(
                    WarpMouseToFocus {
                        mode: None,
                    },
                ),
                focus_follows_mouse: Some(
                    FocusFollowsMouse {
                        max_scroll_amount: None,
                    },
                ),
                workspace_auto_back_and_forth: true,
                mod_key: Some(
                    IsoLevel3Shift,
                ),
                mod_key_nested: Some(
                    Super,
                ),
            },
            outputs: Outputs(
                [
                    Output {
                        off: false,
                        name: "eDP-1",
                        scale: Some(
                            FloatOrInt(
                                2.0,
                            ),
                        ),
                        transform: Flipped90,
                        position: Some(
                            Position {
                                x: 10,
                                y: 20,
                            },
                        ),
                        mode: Some(
                            Mode {
                                custom: false,
                                mode: ConfiguredMode {
                                    width: 1920,
                                    height: 1080,
                                    refresh: Some(
                                        144.0,
                                    ),
                                },
                            },
                        ),
                        modeline: None,
                        variable_refresh_rate: Some(
                            Vrr {
                                on_demand: true,
                            },
                        ),
                        focus_at_startup: true,
                        background_color: Some(
                            Color {
                                r: 0.09803922,
                                g: 0.09803922,
                                b: 0.4,
                                a: 1.0,
                            },
                        ),
                        backdrop_color: None,
                        hot_corners: Some(
                            HotCorners {
                                off: true,
                                top_left: true,
                                top_right: true,
                                bottom_left: true,
                                bottom_right: true,
                            },
                        ),
                        layout: None,
                    },
                    Output {
                        off: false,
                        name: "eDP-2",
                        scale: None,
                        transform: Normal,
                        position: None,
                        mode: Some(
                            Mode {
                                custom: true,
                                mode: ConfiguredMode {
                                    width: 1920,
                                    height: 1080,
                                    refresh: Some(
                                        144.0,
                                    ),
                                },
                            },
                        ),
                        modeline: None,
                        variable_refresh_rate: None,
                        focus_at_startup: false,
                        background_color: None,
                        backdrop_color: None,
                        hot_corners: None,
                        layout: None,
                    },
                    Output {
                        off: false,
                        name: "eDP-3",
                        scale: None,
                        transform: Normal,
                        position: None,
                        mode: None,
                        modeline: Some(
                            Modeline {
                                clock: 173.0,
                                hdisplay: 1920,
                                hsync_start: 2048,
                                hsync_end: 2248,
                                htotal: 2576,
                                vdisplay: 1080,
                                vsync_start: 1083,
                                vsync_end: 1088,
                                vtotal: 1120,
                                hsync_polarity: NHSync,
                                vsync_polarity: PVSync,
                            },
                        ),
                        variable_refresh_rate: None,
                        focus_at_startup: false,
                        background_color: None,
                        backdrop_color: None,
                        hot_corners: None,
                        layout: None,
                    },
                ],
            ),
            spawn_at_startup: [
                SpawnAtStartup {
                    command: [
                        "alacritty",
                        "-e",
                        "fish",
                    ],
                },
            ],
            spawn_sh_at_startup: [
                SpawnShAtStartup {
                    command: "qs -c ~/source/qs/MyAwesomeShell",
                },
            ],
            layout: Layout {
                focus_ring: FocusRing {
                    off: false,
                    width: 5.0,
                    active_color: Color {
                        r: 0.0,
                        g: 0.39215687,
                        b: 0.78431374,
                        a: 1.0,
                    },
                    inactive_color: Color {
                        r: 1.0,
                        g: 0.78431374,
                        b: 0.39215687,
                        a: 0.0,
                    },
                    urgent_color: Color {
                        r: 0.60784316,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    active_gradient: Some(
                        Gradient {
                            from: Color {
                                r: 0.039215688,
                                g: 0.078431375,
                                b: 0.11764706,
                                a: 1.0,
                            },
                            to: Color {
                                r: 0.0,
                                g: 0.5019608,
                                b: 1.0,
                                a: 1.0,
                            },
                            angle: 180,
                            relative_to: WorkspaceView,
                            in_: GradientInterpolation {
                                color_space: Srgb,
                                hue_interpolation: Shorter,
                            },
                        },
                    ),
                    inactive_gradient: None,
                    urgent_gradient: None,
                },
                border: Border {
                    off: false,
                    width: 3.0,
                    active_color: Color {
                        r: 1.0,
                        g: 0.78431374,
                        b: 0.49803922,
                        a: 1.0,
                    },
                    inactive_color: Color {
                        r: 1.0,
                        g: 0.78431374,
                        b: 0.39215687,
                        a: 0.0,
                    },
                    urgent_color: Color {
                        r: 0.60784316,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    active_gradient: None,
                    inactive_gradient: None,
                    urgent_gradient: None,
                },
                shadow: Shadow {
                    on: false,
                    offset: ShadowOffset {
                        x: FloatOrInt(
                            10.0,
                        ),
                        y: FloatOrInt(
                            -20.0,
                        ),
                    },
                    softness: 30.0,
                    spread: 5.0,
                    draw_behind_window: false,
                    color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.46666667,
                    },
                    inactive_color: None,
                },
                tab_indicator: TabIndicator {
                    off: false,
                    hide_when_single_tab: false,
                    place_within_column: false,
                    gap: 5.0,
                    width: 10.0,
                    length: TabIndicatorLength {
                        total_proportion: Some(
                            0.5,
                        ),
                    },
                    position: Top,
                    gaps_between_tabs: 0.0,
                    corner_radius: 0.0,
                    active_color: None,
                    inactive_color: None,
                    urgent_color: None,
                    active_gradient: None,
                    inactive_gradient: None,
                    urgent_gradient: None,
                },
                insert_hint: InsertHint {
                    off: false,
                    color: Color {
                        r: 1.0,
                        g: 0.78431374,
                        b: 0.49803922,
                        a: 1.0,
                    },
                    gradient: Some(
                        Gradient {
                            from: Color {
                                r: 0.039215688,
                                g: 0.078431375,
                                b: 0.11764706,
                                a: 1.0,
                            },
                            to: Color {
                                r: 0.0,
                                g: 0.5019608,
                                b: 1.0,
                                a: 1.0,
                            },
                            angle: 180,
                            relative_to: WorkspaceView,
                            in_: GradientInterpolation {
                                color_space: Srgb,
                                hue_interpolation: Shorter,
                            },
                        },
                    ),
                },
                preset_column_widths: [
                    Proportion(
                        0.25,
                    ),
                    Proportion(
                        0.5,
                    ),
                    Fixed(
                        960,
                    ),
                    Fixed(
                        1280,
                    ),
                ],
                default_column_width: Some(
                    Proportion(
                        0.25,
                    ),
                ),
                preset_window_heights: [
                    Proportion(
                        0.25,
                    ),
                    Proportion(
                        0.5,
                    ),
                    Fixed(
                        960,
                    ),
                    Fixed(
                        1280,
                    ),
                ],
                center_focused_column: OnOverflow,
                always_center_single_column: false,
                empty_workspace_above_first: false,
                default_column_display: Tabbed,
                gaps: 8.0,
                struts: Struts {
                    left: FloatOrInt(
                        1.0,
                    ),
                    right: FloatOrInt(
                        2.0,
                    ),
                    top: FloatOrInt(
                        3.0,
                    ),
                    bottom: FloatOrInt(
                        0.0,
                    ),
                },
                background_color: Color {
                    r: 0.25,
                    g: 0.25,
                    b: 0.25,
                    a: 1.0,
                },
            },
            prefer_no_csd: true,
            cursor: Cursor {
                xcursor_theme: "breeze_cursors",
                xcursor_size: 16,
                hide_when_typing: true,
                hide_after_inactive_ms: Some(
                    3000,
                ),
            },
            screenshot_path: ScreenshotPath(
                Some(
                    "~/Screenshots/screenshot.png",
                ),
            ),
            clipboard: Clipboard {
                disable_primary: true,
            },
            hotkey_overlay: HotkeyOverlay {
                skip_at_startup: true,
                hide_not_bound: false,
            },
            config_notification: ConfigNotification {
                disable_failed: false,
            },
            animations: Animations {
                off: false,
                slowdown: 2.0,
                workspace_switch: WorkspaceSwitchAnim(
                    Animation {
                        off: false,
                        kind: Spring(
                            SpringParams {
                                damping_ratio: 1.0,
                                stiffness: 1000,
                                epsilon: 0.0001,
                            },
                        ),
                    },
                ),
                window_open: WindowOpenAnim {
                    anim: Animation {
                        off: true,
                        kind: Easing(
                            EasingParams {
                                duration_ms: 150,
                                curve: EaseOutExpo,
                            },
                        ),
                    },
                    custom_shader: None,
                },
                window_close: WindowCloseAnim {
                    anim: Animation {
                        off: false,
                        kind: Easing(
                            EasingParams {
                                duration_ms: 150,
                                curve: CubicBezier(
                                    0.05,
                                    0.7,
                                    0.1,
                                    1.0,
                                ),
                            },
                        ),
                    },
                    custom_shader: None,
                },
                horizontal_view_movement: HorizontalViewMovementAnim(
                    Animation {
                        off: false,
                        kind: Easing(
                            EasingParams {
                                duration_ms: 100,
                                curve: EaseOutExpo,
                            },
                        ),
                    },
                ),
                window_movement: WindowMovementAnim(
                    Animation {
                        off: false,
                        kind: Spring(
                            SpringParams {
                                damping_ratio: 1.0,
                                stiffness: 800,
                                epsilon: 0.0001,
                            },
                        ),
                    },
                ),
                window_resize: WindowResizeAnim {
                    anim: Animation {
                        off: false,
                        kind: Spring(
                            SpringParams {
                                damping_ratio: 1.0,
                                stiffness: 800,
                                epsilon: 0.0001,
                            },
                        ),
                    },
                    custom_shader: None,
                },
                config_notification_open_close: ConfigNotificationOpenCloseAnim(
                    Animation {
                        off: false,
                        kind: Spring(
                            SpringParams {
                                damping_ratio: 0.6,
                                stiffness: 1000,
                                epsilon: 0.001,
                            },
                        ),
                    },
                ),
                exit_confirmation_open_close: ExitConfirmationOpenCloseAnim(
                    Animation {
                        off: false,
                        kind: Spring(
                            SpringParams {
                                damping_ratio: 0.6,
                                stiffness: 500,
                                epsilon: 0.01,
                            },
                        ),
                    },
                ),
                screenshot_ui_open: ScreenshotUiOpenAnim(
                    Animation {
                        off: false,
                        kind: Easing(
                            EasingParams {
                                duration_ms: 200,
                                curve: EaseOutQuad,
                            },
                        ),
                    },
                ),
                overview_open_close: OverviewOpenCloseAnim(
                    Animation {
                        off: false,
                        kind: Spring(
                            SpringParams {
                                damping_ratio: 1.0,
                                stiffness: 800,
                                epsilon: 0.0001,
                            },
                        ),
                    },
                ),
                recent_windows_close: RecentWindowsCloseAnim(
                    Animation {
                        off: true,
                        kind: Spring(
                            SpringParams {
                                damping_ratio: 1.0,
                                stiffness: 800,
                                epsilon: 0.001,
                            },
                        ),
                    },
                ),
                material_signal: MaterialSignalAnim(
                    Animation {
                        off: false,
                        kind: Easing(
                            EasingParams {
                                duration_ms: 400,
                                curve: EaseOutCubic,
                            },
                        ),
                    },
                ),
            },
            blur: Blur {
                off: false,
                passes: 3,
                offset: 3.0,
                noise: 0.02,
                saturation: 1.5,
            },
            signal: Signal {
                motion: Full,
            },
            gestures: Gestures {
                dnd_edge_view_scroll: DndEdgeViewScroll {
                    trigger_width: 10.0,
                    delay_ms: 100,
                    max_speed: 50.0,
                },
                dnd_edge_workspace_switch: DndEdgeWorkspaceSwitch {
                    trigger_height: 50.0,
                    delay_ms: 100,
                    max_speed: 1500.0,
                },
                hot_corners: HotCorners {
                    off: false,
                    top_left: false,
                    top_right: false,
                    bottom_left: false,
                    bottom_right: false,
                },
            },
            overview: Overview {
                zoom: 0.5,
                backdrop_color: Color {
                    r: 0.15,
                    g: 0.15,
                    b: 0.15,
                    a: 1.0,
                },
                workspace_shadow: WorkspaceShadow {
                    off: false,
                    offset: ShadowOffset {
                        x: FloatOrInt(
                            0.0,
                        ),
                        y: FloatOrInt(
                            10.0,
                        ),
                    },
                    softness: 40.0,
                    spread: 10.0,
                    color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.3137255,
                    },
                },
            },
            environment: Environment(
                [
                    EnvironmentVariable {
                        name: "QT_QPA_PLATFORM",
                        value: Some(
                            "wayland",
                        ),
                    },
                    EnvironmentVariable {
                        name: "DISPLAY",
                        value: None,
                    },
                ],
            ),
            xwayland_satellite: XwaylandSatellite {
                off: false,
                path: "xwayland-satellite",
            },
            window_rules: [
                WindowRule {
                    matches: [
                        Match {
                            app_id: Some(
                                RegexEq(
                                    Regex(
                                        ".*alacritty",
                                    ),
                                ),
                            ),
                            title: None,
                            is_active: None,
                            is_focused: None,
                            is_active_in_column: None,
                            is_floating: None,
                            is_window_cast_target: None,
                            is_urgent: None,
                            signal_source: None,
                            signal_tag: None,
                            at_startup: None,
                        },
                    ],
                    excludes: [
                        Match {
                            app_id: None,
                            title: Some(
                                RegexEq(
                                    Regex(
                                        "~",
                                    ),
                                ),
                            ),
                            is_active: None,
                            is_focused: None,
                            is_active_in_column: None,
                            is_floating: None,
                            is_window_cast_target: None,
                            is_urgent: None,
                            signal_source: None,
                            signal_tag: None,
                            at_startup: None,
                        },
                        Match {
                            app_id: None,
                            title: None,
                            is_active: Some(
                                true,
                            ),
                            is_focused: Some(
                                false,
                            ),
                            is_active_in_column: None,
                            is_floating: None,
                            is_window_cast_target: None,
                            is_urgent: None,
                            signal_source: None,
                            signal_tag: None,
                            at_startup: None,
                        },
                    ],
                    default_column_width: None,
                    default_window_height: Some(
                        DefaultPresetSize(
                            Some(
                                Fixed(
                                    500,
                                ),
                            ),
                        ),
                    ),
                    open_on_output: Some(
                        "eDP-1",
                    ),
                    open_on_workspace: None,
                    open_maximized: Some(
                        true,
                    ),
                    open_maximized_to_edges: None,
                    open_fullscreen: Some(
                        false,
                    ),
                    open_floating: Some(
                        false,
                    ),
                    open_focused: Some(
                        true,
                    ),
                    min_width: None,
                    min_height: None,
                    max_width: None,
                    max_height: None,
                    focus_ring: BorderRule {
                        off: true,
                        on: false,
                        width: Some(
                            FloatOrInt(
                                3.0,
                            ),
                        ),
                        active_color: None,
                        inactive_color: None,
                        urgent_color: None,
                        active_gradient: None,
                        inactive_gradient: None,
                        urgent_gradient: None,
                    },
                    border: BorderRule {
                        off: false,
                        on: true,
                        width: Some(
                            FloatOrInt(
                                8.5,
                            ),
                        ),
                        active_color: None,
                        inactive_color: None,
                        urgent_color: None,
                        active_gradient: None,
                        inactive_gradient: None,
                        urgent_gradient: None,
                    },
                    shadow: ShadowRule {
                        off: false,
                        on: false,
                        offset: None,
                        softness: None,
                        spread: None,
                        draw_behind_window: None,
                        color: None,
                        inactive_color: None,
                    },
                    tab_indicator: TabIndicatorRule {
                        active_color: Some(
                            Color {
                                r: 1.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            },
                        ),
                        inactive_color: None,
                        urgent_color: None,
                        active_gradient: None,
                        inactive_gradient: None,
                        urgent_gradient: None,
                    },
                    draw_border_with_background: None,
                    opacity: None,
                    material: None,
                    geometry_corner_radius: None,
                    clip_to_geometry: None,
                    baba_is_float: None,
                    block_out_from: None,
                    variable_refresh_rate: None,
                    default_column_display: Some(
                        Tabbed,
                    ),
                    default_floating_position: Some(
                        FloatingPosition {
                            x: FloatOrInt(
                                100.0,
                            ),
                            y: FloatOrInt(
                                -200.0,
                            ),
                            relative_to: BottomLeft,
                        },
                    ),
                    scroll_factor: None,
                    tiled_state: None,
                    background_effect: BackgroundEffectRule {
                        xray: None,
                        blur: None,
                        noise: None,
                        saturation: None,
                    },
                    popups: PopupsRule {
                        opacity: None,
                        geometry_corner_radius: None,
                        background_effect: BackgroundEffectRule {
                            xray: None,
                            blur: None,
                            noise: None,
                            saturation: None,
                        },
                    },
                },
            ],
            materials: [],
            layer_rules: [
                LayerRule {
                    matches: [
                        Match {
                            namespace: Some(
                                RegexEq(
                                    Regex(
                                        "^notifications$",
                                    ),
                                ),
                            ),
                            at_startup: None,
                            layer: None,
                        },
                    ],
                    excludes: [],
                    opacity: None,
                    block_out_from: Some(
                        Screencast,
                    ),
                    shadow: ShadowRule {
                        off: false,
                        on: false,
                        offset: None,
                        softness: None,
                        spread: None,
                        draw_behind_window: None,
                        color: None,
                        inactive_color: None,
                    },
                    geometry_corner_radius: None,
                    place_within_backdrop: None,
                    baba_is_float: None,
                    background_effect: BackgroundEffectRule {
                        xray: None,
                        blur: None,
                        noise: None,
                        saturation: None,
                    },
                    popups: PopupsRule {
                        opacity: None,
                        geometry_corner_radius: None,
                        background_effect: BackgroundEffectRule {
                            xray: None,
                            blur: None,
                            noise: None,
                            saturation: None,
                        },
                    },
                },
            ],
            binds: Binds(
                [
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_Escape,
                            ),
                            modifiers: Modifiers(
                                COMPOSITOR,
                            ),
                        },
                        action: ToggleKeyboardShortcutsInhibit,
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: false,
                        hotkey_overlay_title: Some(
                            Some(
                                "Inhibit",
                            ),
                        ),
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_Escape,
                            ),
                            modifiers: Modifiers(
                                SHIFT | COMPOSITOR,
                            ),
                        },
                        action: ToggleKeyboardShortcutsInhibit,
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: false,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_t,
                            ),
                            modifiers: Modifiers(
                                COMPOSITOR,
                            ),
                        },
                        action: Spawn(
                            [
                                "alacritty",
                            ],
                        ),
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: true,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_q,
                            ),
                            modifiers: Modifiers(
                                COMPOSITOR,
                            ),
                        },
                        action: CloseWindow,
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: Some(
                            None,
                        ),
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_h,
                            ),
                            modifiers: Modifiers(
                                SHIFT | COMPOSITOR,
                            ),
                        },
                        action: FocusMonitorLeft,
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_o,
                            ),
                            modifiers: Modifiers(
                                SHIFT | COMPOSITOR,
                            ),
                        },
                        action: FocusMonitor(
                            "eDP-1",
                        ),
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_l,
                            ),
                            modifiers: Modifiers(
                                CTRL | SHIFT | COMPOSITOR,
                            ),
                        },
                        action: MoveWindowToMonitorRight,
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_o,
                            ),
                            modifiers: Modifiers(
                                CTRL | ALT | COMPOSITOR,
                            ),
                        },
                        action: MoveWindowToMonitor(
                            "eDP-1",
                        ),
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_p,
                            ),
                            modifiers: Modifiers(
                                CTRL | ALT | COMPOSITOR,
                            ),
                        },
                        action: MoveColumnToMonitor(
                            "DP-1",
                        ),
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_comma,
                            ),
                            modifiers: Modifiers(
                                COMPOSITOR,
                            ),
                        },
                        action: ConsumeWindowIntoColumn,
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_1,
                            ),
                            modifiers: Modifiers(
                                COMPOSITOR,
                            ),
                        },
                        action: FocusWorkspace(
                            Index(
                                1,
                            ),
                        ),
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_1,
                            ),
                            modifiers: Modifiers(
                                SHIFT | COMPOSITOR,
                            ),
                        },
                        action: FocusWorkspace(
                            Name(
                                "workspace-1",
                            ),
                        ),
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_e,
                            ),
                            modifiers: Modifiers(
                                SHIFT | COMPOSITOR,
                            ),
                        },
                        action: Quit(
                            true,
                        ),
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: false,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: WheelScrollDown,
                            modifiers: Modifiers(
                                COMPOSITOR,
                            ),
                        },
                        action: FocusWorkspaceDown,
                        repeat: true,
                        cooldown: Some(
                            150ms,
                        ),
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_s,
                            ),
                            modifiers: Modifiers(
                                ALT | SUPER,
                            ),
                        },
                        action: SpawnSh(
                            "pkill orca || exec orca",
                        ),
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: true,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                ],
            ),
            switch_events: SwitchBinds {
                lid_open: None,
                lid_close: None,
                tablet_mode_on: Some(
                    SwitchAction {
                        spawn: [
                            "bash",
                            "-c",
                            "gsettings set org.gnome.desktop.a11y.applications screen-keyboard-enabled true",
                        ],
                    },
                ),
                tablet_mode_off: Some(
                    SwitchAction {
                        spawn: [
                            "bash",
                            "-c",
                            "gsettings set org.gnome.desktop.a11y.applications screen-keyboard-enabled false",
                        ],
                    },
                ),
            },
            debug: Debug {
                preview_render: None,
                dbus_interfaces_in_non_session_instances: false,
                wait_for_frame_completion_before_queueing: false,
                enable_overlay_planes: false,
                disable_cursor_plane: false,
                disable_direct_scanout: false,
                keep_max_bpc_unchanged: false,
                restrict_primary_scanout_to_matching_format: false,
                force_disable_connectors_on_resume: false,
                render_drm_device: Some(
                    "/dev/dri/renderD129",
                ),
                ignored_drm_devices: [
                    "/dev/dri/renderD128",
                    "/dev/dri/renderD130",
                ],
                force_pipewire_invalid_modifier: false,
                emulate_zero_presentation_time: false,
                disable_resize_throttling: false,
                disable_transactions: false,
                keep_laptop_panel_on_when_lid_is_closed: false,
                disable_monitor_names: false,
                strict_new_window_focus_policy: false,
                honor_xdg_activation_with_invalid_serial: false,
                deactivate_unfocused_windows: false,
                skip_cursor_only_updates_during_vrr: false,
            },
            workspaces: [
                Workspace {
                    name: WorkspaceName(
                        "workspace-1",
                    ),
                    open_on_output: Some(
                        "eDP-1",
                    ),
                    layout: None,
                },
                Workspace {
                    name: WorkspaceName(
                        "workspace-2",
                    ),
                    open_on_output: None,
                    layout: None,
                },
                Workspace {
                    name: WorkspaceName(
                        "workspace-3",
                    ),
                    open_on_output: None,
                    layout: None,
                },
            ],
            recent_windows: RecentWindows {
                on: false,
                debounce_ms: 750,
                open_delay_ms: 150,
                highlight: MruHighlight {
                    active_color: Color {
                        r: 0.0,
                        g: 1.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    urgent_color: Color {
                        r: 1.0,
                        g: 0.6,
                        b: 0.6,
                        a: 1.0,
                    },
                    padding: 15.0,
                    corner_radius: 0.0,
                },
                previews: MruPreviews {
                    max_height: 960.0,
                    max_scale: 0.5,
                },
                binds: [
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_Tab,
                            ),
                            modifiers: Modifiers(
                                ALT,
                            ),
                        },
                        action: MruAdvance {
                            direction: Forward,
                            scope: None,
                            filter: Some(
                                All,
                            ),
                        },
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_grave,
                            ),
                            modifiers: Modifiers(
                                ALT,
                            ),
                        },
                        action: MruAdvance {
                            direction: Forward,
                            scope: None,
                            filter: Some(
                                AppId,
                            ),
                        },
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                    Bind {
                        key: Key {
                            trigger: Keysym(
                                XK_Tab,
                            ),
                            modifiers: Modifiers(
                                SUPER,
                            ),
                        },
                        action: MruAdvance {
                            direction: Forward,
                            scope: Some(
                                Output,
                            ),
                            filter: Some(
                                All,
                            ),
                        },
                        repeat: true,
                        cooldown: None,
                        allow_when_locked: false,
                        allow_inhibiting: true,
                        hotkey_overlay_title: None,
                    },
                ],
            },
        }
        "#);
    }

    fn diff_lines(expected: &str, actual: &str) -> String {
        let mut output = String::new();
        let mut in_change = false;

        for change in diff::lines(expected, actual) {
            match change {
                diff::Result::Both(_, _) => {
                    in_change = false;
                }
                diff::Result::Left(line) => {
                    if !output.is_empty() && !in_change {
                        output.push('\n');
                    }
                    output.push('-');
                    output.push_str(line);
                    output.push('\n');
                    in_change = true;
                }
                diff::Result::Right(line) => {
                    if !output.is_empty() && !in_change {
                        output.push('\n');
                    }
                    output.push('+');
                    output.push_str(line);
                    output.push('\n');
                    in_change = true;
                }
            }
        }

        output
    }

    #[test]
    fn diff_empty_to_default() {
        // We try to write the config defaults in such a way that empty sections (and an empty
        // config) give the same outcome as the default config bundled with niri. This test
        // verifies the actual differences between the two.
        let mut default_config = Config::load_default();
        let empty_config = Config::parse_mem("").unwrap();

        // Some notable omissions: the default config has some window rules, and an empty config
        // will not have any binds. Clear them out so they don't spam the diff.
        default_config.window_rules.clear();
        default_config.binds.0.clear();

        assert_snapshot!(
            diff_lines(
                &format!("{empty_config:#?}"),
                &format!("{default_config:#?}")
            ),
            @r#"
        -            numlock: false,
        +            numlock: true,

        -            tap: false,
        +            tap: true,

        -            natural_scroll: false,
        +            natural_scroll: true,

        -    spawn_at_startup: [],
        +    spawn_at_startup: [
        +        SpawnAtStartup {
        +            command: [
        +                "waybar",
        +            ],
        +        },
        +    ],

        -                0.3333333333333333,
        +                0.33333,

        -                0.6666666666666666,
        +                0.66667,
        "#,
        );
    }
}
