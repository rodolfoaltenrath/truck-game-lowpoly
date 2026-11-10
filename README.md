# Jogo de caminhão 3D para navegador

Protótipo de um jogo original de caminhão em 3D, multiplayer, que roda no
navegador. Cliente em TypeScript (Babylon.js + Rapier), servidor e núcleo
compartilhado em Rust.

Plano completo: [ROADMAP_JOGO_WEB.md](ROADMAP_JOGO_WEB.md).

## Rodar

```
cd web
pnpm install
pnpm dev
```

Abra http://localhost:5173. Outros PCs da rede local usam o IP desta máquina
na mesma porta. `?webgpu` na URL tenta WebGPU (padrão: WebGL2).

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
crates/game_core/   núcleo Rust sem I/O (nativo + WASM)
web/                cliente Vite + TypeScript
  src/vehicle/      caminhão, trem de força
  src/world/        pista de testes procedural
  e2e/              testes Playwright
legacy/mod-18wos/   código do mod antigo, fonte para extração
```
