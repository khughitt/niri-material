//! Scratch probe (material-03fdf3): dump the material program's driver binary.
use smithay::backend::egl::{EGLContext, EGLDisplay};
use smithay::backend::egl::native::EGLSurfacelessDisplay;
use smithay::backend::renderer::gles::{ffi, link_program, GlesRenderer};

#[test]
#[ignore]
fn dump_material_program_binary() {
    let out = std::env::var("PROBE_OUT").unwrap();
    let src = std::env::var("PROBE_SRC")
        .map(|p| std::fs::read_to_string(p).unwrap())
        .unwrap_or_else(|_| crate::render_helpers::shaders::material_source());
    std::fs::write(format!("{out}.frag"), &src).unwrap();
    let mut renderer = unsafe {
        let display = EGLDisplay::new(EGLSurfacelessDisplay).unwrap();
        let context = EGLContext::new(&display).unwrap();
        GlesRenderer::new(context).unwrap()
    };
    renderer
        .with_context(|gl| unsafe {
            let shader = format!("#version 100\n{src}");
            let program = link_program(gl, include_str!("../render_helpers/shaders/texture.vert"), &shader).unwrap();
            let mut len = 0;
            gl.GetProgramiv(program, ffi::PROGRAM_BINARY_LENGTH, &mut len);
            let mut buf = vec![0u8; len as usize];
            let mut written = 0;
            let mut format = 0;
            gl.GetProgramBinary(program, len, &mut written, &mut format, buf.as_mut_ptr().cast());
            buf.truncate(written as usize);
            eprintln!("binary format {format:#x}, {written} bytes");
            std::fs::write(format!("{out}.bin"), &buf).unwrap();
        })
        .unwrap();
}
