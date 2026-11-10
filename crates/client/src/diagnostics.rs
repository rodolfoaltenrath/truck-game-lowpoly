use fyrox::{engine::InitializedGraphicsContext, graphics_gl::server::GlGraphicsServer};
use glow::HasContext;
pub fn gpu(graphics: &InitializedGraphicsContext) -> String {
    let Some(server) = graphics
        .renderer
        .graphics_server()
        .as_any()
        .downcast_ref::<GlGraphicsServer>()
    else {
        return "OpenGL (identificação indisponível)".into();
    };
    // Callback executado na thread da janela, com o contexto OpenGL corrente.
    unsafe {
        format!(
            "{} | {} | {}",
            server.gl.get_parameter_string(glow::VENDOR),
            server.gl.get_parameter_string(glow::RENDERER),
            server.gl.get_parameter_string(glow::VERSION)
        )
    }
}
pub fn capture(
    graphics: &InitializedGraphicsContext,
    path: &std::path::Path,
) -> anyhow::Result<()> {
    let server = graphics
        .renderer
        .graphics_server()
        .as_any()
        .downcast_ref::<GlGraphicsServer>()
        .ok_or_else(|| anyhow::anyhow!("captura requer OpenGL"))?;
    let size = graphics.window.inner_size();
    anyhow::ensure!(
        size.width > 0 && size.height > 0 && size.width <= 7680 && size.height <= 4320,
        "tamanho de captura inválido"
    );
    let mut bytes = vec![0u8; size.width as usize * size.height as usize * 4];
    // Lê o último quadro apresentado. Restaura o estado OpenGL alterado para
    // não interferir no cache do renderizador nem capturar outras janelas.
    unsafe {
        let gl = &server.gl;
        let old_fbo = gl.get_parameter_i32(glow::READ_FRAMEBUFFER_BINDING) as u32;
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        let old_buffer = gl.get_parameter_i32(glow::READ_BUFFER) as u32;
        let old_pack = gl.get_parameter_i32(glow::PACK_ALIGNMENT);
        gl.read_buffer(glow::FRONT);
        gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
        gl.read_pixels(
            0,
            0,
            size.width as i32,
            size.height as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(&mut bytes)),
        );
        gl.pixel_store_i32(glow::PACK_ALIGNMENT, old_pack);
        gl.read_buffer(old_buffer);
        gl.bind_framebuffer(
            glow::READ_FRAMEBUFFER,
            std::num::NonZeroU32::new(old_fbo).map(glow::NativeFramebuffer),
        );
    }
    let mut image = image::RgbaImage::from_raw(size.width, size.height, bytes)
        .ok_or_else(|| anyhow::anyhow!("buffer de captura inválido"))?;
    image::imageops::flip_vertical_in_place(&mut image);
    image.save(path)?;
    Ok(())
}
