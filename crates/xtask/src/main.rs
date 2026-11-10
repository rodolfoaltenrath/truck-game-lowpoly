//! Ferramentas Rust de distribuição; sem navegador, gerenciador JS ou scripts auxiliares.
use anyhow::{Context, Result, bail};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        bail!("comando falhou: {command:?}");
    }
    Ok(())
}
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) != Some("dist")
        || args.len() > 2
        || args.get(1).is_some_and(|s| s != "--dev")
    {
        bail!("uso: cargo run -p xtask -- dist [--dev]");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    std::env::set_current_dir(root)?;
    let dev = args.get(1).is_some();
    let profile = if dev { "debug" } else { "release" };
    let mut build = Command::new("cargo");
    build.args([
        "build",
        "--locked",
        "-p",
        "truck_client",
        "-p",
        "truck_server",
    ]);
    if !dev {
        build.arg("--release");
    }
    run(&mut build)?;
    let target = Command::new("rustc").arg("-vV").output()?;
    let text = String::from_utf8(target.stdout)?;
    let host = text
        .lines()
        .find_map(|l| l.strip_prefix("host: "))
        .context("host Rust ausente")?;
    let name = format!(
        "sul-do-brasil-{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let dist = root.join("dist");
    let package = dist.join(&name);
    std::fs::create_dir_all(&package)?;
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    let target_dir = if let Some(target) = std::env::var_os("CARGO_BUILD_TARGET") {
        target_dir.join(target)
    } else {
        target_dir
    };
    for binary in ["truck_client", "truck_server"] {
        let file = format!("{binary}{}", std::env::consts::EXE_SUFFIX);
        std::fs::copy(target_dir.join(profile).join(&file), package.join(&file))?;
        if dev && host.ends_with("windows-gnu") {
            run(Command::new("strip")
                .arg("--strip-debug")
                .arg(package.join(&file)))?;
        }
    }
    for file in [
        "README.md",
        "LICENSE",
        "ROADMAP_JOGO_NATIVO.md",
        "ROADMAP_JOGO_WEB.md",
    ] {
        std::fs::copy(root.join(file), package.join(file))?;
    }
    std::fs::create_dir_all(package.join("docs"))?;
    for file in ["CONTENT_LICENSES.md", "MAPA_SUL.md", "WINDOWS_DEV.md"] {
        std::fs::copy(
            root.join("docs").join(file),
            package.join("docs").join(file),
        )?;
    }
    copy_notices(&root.join("licenses"), &package.join("licenses"), 0)?;
    let sysroot = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?;
    let rust_docs = PathBuf::from(String::from_utf8(sysroot.stdout)?.trim()).join("share/doc/rust");
    if rust_docs.exists() {
        let destination = package.join("licenses/rust");
        std::fs::create_dir_all(&destination)?;
        for file in ["COPYRIGHT.html", "COPYRIGHT-library.html"] {
            let source = rust_docs.join(file);
            if source.is_file() {
                std::fs::copy(source, destination.join(file))?;
            }
        }
        if rust_docs.join("licenses").is_dir() {
            copy_notices(
                &rust_docs.join("licenses"),
                &destination.join("licenses"),
                0,
            )?;
        }
    }
    let metadata = Command::new("cargo")
        .args([
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--filter-platform",
            host,
        ])
        .output()?;
    if !metadata.status.success() {
        bail!("cargo metadata falhou");
    }
    let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout)?;
    let mut index =
        String::from("Dependências Rust; textos de licença nas subpastas correspondentes.\n\n");
    for p in metadata["packages"]
        .as_array()
        .context("pacotes ausentes")?
    {
        if p["source"].is_null() {
            continue;
        }
        let name = p["name"].as_str().context("nome ausente")?;
        let version = p["version"].as_str().context("versão ausente")?;
        let source = Path::new(p["manifest_path"].as_str().context("manifesto ausente")?)
            .parent()
            .context("pasta ausente")?;
        let destination = package
            .join("licenses/dependencies")
            .join(format!("{name}-{version}"));
        copy_notices(source, &destination, 0)?;
        index.push_str(&format!(
            "{name} {version} | {} | https://crates.io/crates/{name}/{version}\n",
            p["license"].as_str().unwrap_or("consultar pacote")
        ));
    }
    std::fs::write(package.join("licenses/DEPENDENCIES.txt"), index)?;
    let archive = dist.join(format!(
        "{name}.{}",
        if cfg!(windows) { "zip" } else { "tar.gz" }
    ));
    let mut tar = Command::new("tar");
    tar.arg(if cfg!(windows) { "-acf" } else { "-czf" })
        .arg(&archive)
        .arg("-C")
        .arg(&dist)
        .arg(&name);
    run(&mut tar)?;
    println!(
        "Pacote {}: {}",
        if dev { "de desenvolvimento" } else { "release" },
        archive.display()
    );
    Ok(())
}
fn copy_notices(source: &Path, destination: &Path, depth: u8) -> Result<()> {
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let name = entry.file_name();
        let lower = name.to_string_lossy().to_ascii_lowercase();
        if kind.is_file()
            && (lower.contains("license")
                || lower.contains("licence")
                || lower.starts_with("copying")
                || lower.starts_with("notice")
                || source.file_name().is_some_and(|s| s == "licenses"))
        {
            if entry.metadata()?.len() > 1024 * 1024 {
                bail!("aviso excede 1 MiB: {}", entry.path().display());
            }
            std::fs::create_dir_all(destination)?;
            std::fs::copy(entry.path(), destination.join(name))?;
        } else if kind.is_dir() && depth < 2 && (lower == "licenses" || lower == "license") {
            copy_notices(&entry.path(), &destination.join(name), depth + 1)?;
        }
    }
    Ok(())
}
