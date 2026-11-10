# Orientações para agentes — jogo de caminhão web

Leia `ROADMAP_JOGO_WEB.md` (visão, stack, fases F0–F13 e estado de cada uma)
e `docs/ART_STYLE.md` (estilo low poly, paleta, regras de modelagem) antes de
implementar.

`legacy/mod-18wos/` é uma cópia local, **fora do Git**, do mod antigo para 18
Wheels of Steel (versionado no repositório `rodolfoaltenrath/18wos-across-america-mp`).
Serve só como fonte de código para extrair (protocolo, sessão, interpolação,
economia); se não existir, clonar aquele repositório. Código extraído entra em
`crates/` com nota de origem no topo do arquivo. As
regras e o histórico do `legacy/mod-18wos/AGENTS.md` **não** valem para este
jogo. Não adicionar `legacy/` ao workspace.

## Regras

- Rust para servidor, núcleo compartilhado (`crates/game_core`), regras e
  ferramentas. TypeScript só no cliente web (`web/`). Shaders, HTML/CSS e SQL
  quando necessário. Nenhuma outra linguagem sem decisão registrada.
- Lógica compartilhada não é reescrita em TS: passa por `game_core` em WASM.
- `game_core` não faz I/O (sem `std::fs`, `std::net`, relógio do sistema).
- Conteúdo (modelos, texturas, sons, fontes) só próprio ou com licença que
  permita o uso; registrar origem e licença. Nada de assets de jogos
  comerciais nem de mods derivados deles, nem para teste.
- Limites explícitos em toda entrada de rede/arquivo; recusa tipada.
- Roadmap atualizado com o que foi entregue, como foi testado e o que falta.
  Não marcar como testado por pessoas o que só passou em teste automatizado.
- Repositório: `https://github.com/rodolfoaltenrath/truck-game-lowpoly`
  (nome provisório). Em 09/11/2026 o proprietário autorizou commit e envio
  (push) ao concluir blocos validados, **sempre sem coautor** (nenhum trailer
  `Co-authored-by`). Não inferir autorização para releases ou mensagens
  externas.

## Ambiente (Fedora, sem sudo)

- Node 24 LTS em `~/.local/opt/node` (links em `~/.local/bin`), pnpm 10.
- Rust via rustup em `~/.cargo` (toolchain fixado em `rust-toolchain.toml`,
  alvo `wasm32-unknown-unknown`).
- `gcc` instalado pelo proprietário em 10/11/2026; `wasm-bindgen-cli`
  0.2.130 via `cargo install` (versão precisa bater com o `Cargo.lock`).
- Push por SSH (chave do proprietário, sem senha) para `origin`.

## Fluxo de desenvolvimento

```
cd web
pnpm server       # sala em ws://0.0.0.0:39100 (Rust)
pnpm dev          # gera o WASM e abre http://localhost:5173 com recarga
pnpm check        # tsc + biome
pnpm test:e2e     # Playwright (sobe Vite e servidor se preciso)
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
```

Em modo dev, `window.__game` expõe `telemetry()`, `setInput()` e `teleport()`
para testes e console. A pose do caminhão sobrevive ao recarregamento.
