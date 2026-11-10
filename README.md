# Sul do Brasil — jogo de caminhão nativo

Jogo original low poly em **Rust**, para **Windows e Linux**. Cliente Fyrox/OpenGL,
física Rapier e servidor Rust. Sem Electron, WebView ou navegador no executável.

Direção vigente e pendências: [roadmap nativo](ROADMAP_JOGO_NATIVO.md).
O [mapa do Sul](docs/MAPA_SUL.md), incluindo União da Vitória/Porto União como
polo inicial multiplayer, tem um primeiro circuito piloto. O executável oferece
dois pátios de treino, estrada direta e retorno alternativo. O atlas regional
contém as 32 cidades planejadas; os demais corredores ainda não existem em 3D.

## Executar

Instale Rust com rustup. A primeira compilação baixa e compila a engine e pode
demorar bastante. O jogo compilado não precisa de Rust, Node ou pnpm.

```sh
# Sozinho
cargo run -p truck_client -- --solo

# Multiplayer: servidor em um terminal e um cliente por jogador
cargo run -p truck_server
cargo run -p truck_client -- --server ws://127.0.0.1:39100 --room lan
```

Outro PC usa `--server ws://IP-DO-SERVIDOR:39100`. Liberar a porta TCP 39100 no
firewall da máquina que hospeda a sala. O servidor também pode rodar no Linux
sem ambiente gráfico. Não é necessário abrir localhost:5173.

**Windows:** Rust MSVC com C++ Build Tools ou Rust GNU com MinGW. Para esta
máquina, consultar [WINDOWS_DEV.md](docs/WINDOWS_DEV.md).

Para experimentar alterações nesta máquina, use o ciclo incremental:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1
```

Acrescente `-Check` para verificar o código sem gerar o executável ou abrir a janela.
Não executa testes. Preserve `target/` para reaproveitar o cache de compilação.

Para validar e gerar o pacote nesta máquina, execute na raiz:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-native.ps1 -Smoke
```

O script roda testes e Clippy, gera o pacote em `dist/` e salva captura/relatório
em `target/native-review/`. Sem `-Smoke`, não abre a janela do jogo.

**Linux:** OpenGL 3.3 e bibliotecas de desenvolvimento para compilar:

```sh
# Fedora
sudo dnf install gcc-c++ pkgconf-pkg-config alsa-lib-devel libX11-devel \
  libXcursor-devel libXrandr-devel libXi-devel mesa-libGL-devel \
  libxkbcommon-devel wayland-devel systemd-devel

# Debian/Ubuntu
sudo apt install build-essential pkg-config libasound2-dev libudev-dev \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev \
  libxkbcommon-dev libwayland-dev
```

Os dados ficam no diretório local do usuário (`SulDoBrasil` no Windows,
`suldobrasil` conforme XDG no Linux). Use `--data-dir ./userdata` para guardar
preferências e posição em uma pasta escolhida, inclusive junto ao executável.

## Controles do primeiro port

| Ação | Tecla |
|---|---|
| Acelerar | W / ↑ |
| Frear / ré ao segurar parado | S / ↓ |
| Direção | A/D / ←/→ |
| Freio de mão | Espaço |
| Frete de treino: aceitar/carregar/descarregar | E |
| Mapa local / fechar mapa | M |
| Alternar mapa local e planejamento regional | Tab, com mapa aberto |
| Destino / origem / preferência de rota no atlas regional | ↑/↓ / O / Espaço |
| Engatar/desengatar próximo e parado | T |
| Voltar ao pátio e reiniciar o treino | R |
| Câmera externa, frontal, aérea | C |
| Orbitar câmera externa / zoom | Arrastar botão direito / roda do mouse |
| Pintura | P |
| Sombras ligadas/desligadas | G |
| Mostrar ajuda | H |
| Sair | Esc |

O preset inicial prioriza leveza e nitidez, sem redução automática de resolução.
Gamepad, áudio, outros tipos de reboque e engate automático ainda precisam de
port. Consulte o roadmap antes de considerar equivalentes os marcos web antigos.

## Primeiro frete de treino

Pressione **R** para voltar ao pátio de União da Vitória (também disponível pelo
mapa). Com cavalo e baú parados na área verde, pressione **E** e aguarde 2 segundos.
Abra **M** para se orientar: saia do pátio para oeste e siga para sul até Porto
União; o circuito a leste oferece outro caminho. Pare o conjunto na área verde
do segundo pátio e pressione **E** para descarregar. Após concluir, E prepara
o frete inverso, e outro E inicia a carga.

Mover ou desengatar o conjunto durante carga/descarga cancela a operação. O treino
é local e reinicia ao sair ou usar R; não dá dinheiro nem altera carreira ou estado
do servidor. Os prédios e bombas ainda são decorativos, sem colisões ou serviços
funcionais. O multiplayer continua sincronizando veículos; fretes compartilhados
e reserva de vagas dependem de implementação autoritativa.

## Validar

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p truck_client -- --solo --smoke-frames 180 \
  --report native-smoke.json --data-dir ./target/smoke-userdata
```

O smoke abre uma janela real e grava um relatório; exige sessão gráfica e
OpenGL. Não mede, sozinho, conforto ao dirigir ou multiplayer entre pessoas.

## Pacote portátil

```sh
cargo run -p xtask -- dist
# Ou um pacote de desenvolvimento, sem otimização release:
cargo run -p xtask -- dist --dev
```

O comando compila cliente e servidor para o sistema atual e cria `dist/`,
incluindo documentação e avisos de licença. No Windows, extrair o ZIP e
executar `truck_client.exe`; no Linux, extrair o `.tar.gz` e executar
`./truck_client`. O servidor é opcional para jogar sozinho (`--solo`).
Os pacotes não instalam nem incluem navegador. Linux depende das bibliotecas
do sistema e de um driver OpenGL; a versão mínima de glibc depende do ambiente
em que o pacote foi compilado. O CI gera pacotes separados por plataforma,
sem publicar releases automaticamente.

## Estrutura

```text
crates/client/     cliente Rust nativo, render, teclado, HUD, rede e dados locais
crates/game_sim/   física e trem de força Rust, passo fixo, sem I/O
crates/game_core/  protocolo, sala, sessão, interpolação, seleção de regiões
crates/server/     servidor da sala, Tokio + WebSocket
crates/xtask/      empacotamento e coleta dos avisos de licença
legacy/           fonte de extração do mod antigo, fora do Git/workspace
```

Geometria e paleta são próprias. Fontes e dependências têm suas licenças
registradas em [CONTENT_LICENSES.md](docs/CONTENT_LICENSES.md).


teste em dev: powershell -ExecutionPolicy Bypass -File .\scripts\dev.ps1
