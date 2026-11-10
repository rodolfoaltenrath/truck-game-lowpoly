# Desenvolvimento nativo nesta máquina Windows

O projeto usa Rust 1.99.0, sem Node, pnpm ou wasm-bindgen. Neste computador foi
preparado w64devkit fora do repositório, em `%LOCALAPPDATA%/truck-build-tools`.
Não houve alteração do PATH global.

Em PowerShell, na raiz do repositório:

```powershell
$buildTools = Join-Path $env:LOCALAPPDATA 'truck-build-tools'
$env:Path = "$buildTools/w64devkit/bin;$env:Path"
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = '-Clink-self-contained=yes'
cargo run -p truck_client -- --solo
```

Em outro terminal com as mesmas variáveis, `cargo run -p truck_server` abre a
sala. Para multiplayer, execute o cliente sem `--solo`. Não abrir o antigo
endereço localhost:5173: o cliente agora abre uma janela própria.

Esses caminhos descrevem a preparação local. Uma instalação nova pode usar
Rust MSVC com Visual Studio C++ Build Tools ou Rust GNU com MinGW compatível.
Nenhum binário dessas ferramentas entra no Git ou no pacote do jogo.

O Windows desta máquina informa Intel HD Graphics 4000. A validação do port
deve registrar a execução nativa real, sem confundir resultados de SwiftShader
do protótipo web ou de WSLg com desempenho desse driver OpenGL.
