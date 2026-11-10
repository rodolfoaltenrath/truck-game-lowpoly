# Jogo de caminhão 3D para navegador

Protótipo de um jogo original de caminhão em 3D, multiplayer, que roda no
navegador. Cliente em TypeScript (Babylon.js + Rapier), servidor e núcleo
compartilhado em Rust.

Plano completo: [ROADMAP_JOGO_WEB.md](ROADMAP_JOGO_WEB.md).

## Rodar

Requisitos: Rust (rustup), `gcc`, Node 24 + pnpm e
`cargo install wasm-bindgen-cli --version 0.2.130`.
No ambiente Windows desta máquina, consultar [configuração local](docs/WINDOWS_DEV.md).

```
# terminal 1: sala do multiplayer
cd web && pnpm server            # ws://0.0.0.0:39100, sala "lan"

# terminal 2: jogo
cd web && pnpm install && pnpm dev   # http://localhost:5173
```

Cada aba do navegador é um jogador. Outros PCs da rede local abrem
`http://IP-DESTA-MÁQUINA:5173` e conectam sozinhos na sala desta máquina.
Parâmetros de URL: `?solo` (sem rede), `?servidor=ws://host:39100`,
`?sala=nome`, `?webgpu` (tenta WebGPU; padrão WebGL2). Para testar sozinho
com um "fantasma" seu: `pnpm server --echo`.

Mapa experimental da F4: abrir `http://localhost:5173/?world=road&solo`.
São 12 km de rodovia entre dois povoados, com terreno e colisores carregados
por regiões. A pista de testes continua no endereço original.
Detalhes e próximos passos: [mapa grande](docs/WORLD_STREAMING.md).

## Controles

| Ação | Teclado | Controle |
|---|---|---|
| Acelerar | W / ↑ | RT |
| Frear / ré (segurar parado) | S / ↓ | LT |
| Esterçar | A D / ← → | analógico esquerdo |
| Freio de mão | Espaço | A |
| Trocar câmera | C | Y |
| Endireitar | R | Back |
| Olhar em volta | arrastar o mouse (roda = zoom) | — |
| Engatar reboque | dar ré até encostar | — |
| Desengatar reboque | T (parado) | B |
| Trocar pintura | P | — |
| Qualidade gráfica | G | — |
| Esconder ajuda | H | — |

## Estrutura

```
crates/game_core/   núcleo Rust sem I/O: protocolo, sala, cliente, interpolação
crates/game_core_wasm/  ponte wasm-bindgen para o navegador
crates/server/      servidor da sala (Tokio + WebSocket)
web/                cliente Vite + TypeScript
  src/vehicle/      caminhão, trem de força
  src/world/        pista de testes procedural
  src/net/          WebSocket, sessão (WASM) e caminhões remotos
  e2e/              testes Playwright
legacy/mod-18wos/   código do mod antigo, fonte para extração
```
