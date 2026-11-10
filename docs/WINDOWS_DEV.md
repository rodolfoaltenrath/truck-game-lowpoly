# Desenvolvimento nativo nesta máquina Windows

O projeto usa Rust 1.99.0, sem Node, pnpm ou wasm-bindgen. Neste computador foi
preparado w64devkit fora do repositório, em `%LOCALAPPDATA%/truck-build-tools`.
Não houve alteração do PATH global.

Em PowerShell, na raiz do repositório:

```powershell
# Uso diário: compila incrementalmente apenas o cliente e abre o jogo solo.
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1

# Verifica código e testes do cliente sem gerar/linkar executável; não roda testes.
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1 -Check

# Valida, compila em release e gera o pacote portátil.
powershell -ExecutionPolicy Bypass -File .\scripts\build-native.ps1

# Também executa uma sessão curta e salva captura/relatório em target/native-review.
powershell -ExecutionPolicy Bypass -File .\scripts\build-native.ps1 -Smoke
```

Use `dev.ps1` a cada alteração e reserve `build-native.ps1` para validação completa
e distribuição. Cargo reaproveita dependências em `target/`; alterações no cliente
normalmente recompilam o cliente e refazem a linkedição. A primeira execução ainda
pode demorar. Preserve `target/`, o toolchain e as flags: `cargo clean`, mudanças de
perfil, dependências, target ou flags podem exigir novas compilações. `check` também
mantém seu cache, mas não substitui testes nem confirma a renderização. Mudanças em
geometria Rust exigem recompilar/reabrir; este fluxo não implementa hot reload.

`-Dev` gera um pacote de desenvolvimento; release é recomendado para avaliar
desempenho. A primeira compilação pode demorar. O script não instala ferramentas
nem altera configurações globais. Se uma verificação falhar, ele para antes de
empacotar. A captura exige uma sessão gráfica ativa; examine também o relatório
e jogue manualmente para avaliar aparência, rodas e estabilidade.

Para executar diretamente durante desenvolvimento:

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

Para capturas automatizadas dos mapas, o cliente aceita `--smoke-frames 90
--smoke-map local` ou `--smoke-map regional`, combinados com `--screenshot caminho.png`
e `--report caminho.json`. Esses modos são incompatíveis com `--smoke-drive`.

## Janelas de teste e diagnóstico

`--smoke-frames N` fecha a janela ao completar N quadros, inclusive durante
direção manual. `build-native.ps1 -Smoke` usa 600 quadros; para jogar normalmente,
use `dev.ps1`, que não impõe esse limite. `--smoke-maneuvers` alterna aceleração,
direção, frenagem e ré para reprodução automática e também exige limite de quadros.

No próximo uso do cliente atualizado, `last-session.txt` registra estado recente
e motivo de encerramento. Um panic Rust gera `crash-latest.txt` com backtrace.
Ambos ficam junto de `settings.json`, respeitando `--data-dir`. Na pasta padrão:

```powershell
Get-Content "$env:LOCALAPPDATA/TruckGame/SulDoBrasil/data/last-session.txt"
```

Um arquivo que permanece em "executando" após o processo sumir não identifica a
causa: pode ser encerramento externo ou falha nativa sem panic Rust. O código
de saída e os registros do sistema ainda podem ser necessários nesse caso.
