use fyrox::{engine::InitializedGraphicsContext, graphics_gl::server::GlGraphicsServer};
use glow::HasContext;
/// Arquivos pequenos e sobrescritos no diretório de dados; nada é enviado pela rede.
pub fn marker(message: &str) {
    let bounded: String = message.chars().take(4096).collect();
    let _ = std::fs::write("last-session.txt", bounded);
}
pub fn install_crash_handler() {
    marker("iniciando cliente");
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let report = format!(
            "Rust panic · {} · {}\n{info}\n{}",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::backtrace::Backtrace::force_capture()
        );
        let bounded: String = report.chars().take(32768).collect();
        let _ = std::fs::write("crash-latest.txt", bounded);
        marker("panic Rust: consulte crash-latest.txt");
        previous(info);
    }));
}
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

#[cfg(test)]
mod tests {
    #[test]
    fn rust_panic_leaves_a_bounded_local_report() {
        const CHILD: &str = "TRUCK_CRASH_REPORT_TEST";
        if std::env::var_os(CHILD).is_some() {
            super::install_crash_handler();
            panic!("diagnóstico de teste, não é uma sessão de jogo");
        }
        let folder = std::env::temp_dir().join(format!(
            "truck-crash-report-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&folder).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "diagnostics::tests::rust_panic_leaves_a_bounded_local_report",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .current_dir(&folder)
            .output()
            .unwrap();
        assert!(!output.status.success());
        let report = std::fs::read_to_string(folder.join("crash-latest.txt")).unwrap();
        assert!(report.contains("diagnóstico de teste"));
        assert!(report.len() <= 131072);
        assert!(
            std::fs::read_to_string(folder.join("last-session.txt"))
                .unwrap()
                .contains("panic Rust")
        );
        for file in ["crash-latest.txt", "last-session.txt"] {
            std::fs::remove_file(folder.join(file)).unwrap();
        }
        std::fs::remove_dir(folder).unwrap();
    }
}
