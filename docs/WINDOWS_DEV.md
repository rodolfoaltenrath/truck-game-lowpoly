# Desenvolvimento nesta máquina Windows

O ambiente original do projeto é Fedora. Nesta sessão foram preparados,
sem alteração do PATH global, w64devkit e wasm-bindgen-cli 0.2.130 em
`$env:LOCALAPPDATA/truck-build-tools`. O pnpm é acessado pelo Corepack.

Para usar essas ferramentas em um terminal PowerShell novo, na raiz do repo:

```powershell
$tools = Join-Path $env:LOCALAPPDATA 'truck-build-tools'
$env:Path = "$tools/w64devkit/bin;$tools/wasm-bindgen-0.2.130-x86_64-pc-windows-msvc;$tools;$env:Path"
$sysroot = rustc --print sysroot
$env:LIBRARY_PATH = "$sysroot/lib/rustlib/x86_64-pc-windows-gnu/lib/self-contained"
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = '-Clink-self-contained=yes'
cd web
pnpm.cmd dev
```

Usar `pnpm.cmd` evita depender da política de execução de scripts `.ps1`.
`LIBRARY_PATH` aponta para as bibliotecas de runtime distribuídas com o
toolchain Rust GNU, inclusive para os build scripts nativos do build WASM.
Em outro terminal com as mesmas variáveis, `pnpm.cmd server` abre a sala.

Esses caminhos descrevem as ferramentas locais desta máquina. Em uma
instalação nova, preparar Rust GNU, Node/Corepack, um toolchain MinGW
compatível e wasm-bindgen-cli na versão do `Cargo.lock` antes de executar.
Não há binários dessas ferramentas no repositório.

O Git padroniza texto em LF por `.gitattributes`, para que o checkout no
Windows também passe no formatador Biome.
