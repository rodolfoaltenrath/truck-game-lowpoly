# Orientações para agentes — jogo de caminhão nativo

Leia `ROADMAP_JOGO_NATIVO.md` (decisão de migração, stack e estado das fases)
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

- Em 10/11/2026 o proprietário decidiu migrar para **100% Rust nativo**, em
  Windows e Linux, sem navegador, Electron ou WebView no binário.
- Rust para cliente (`crates/client`, Fyrox/OpenGL), física (`crates/game_sim`),
  servidor, núcleo compartilhado (`crates/game_core`) e ferramentas.
  Shaders e SQL quando necessários. Nenhuma outra linguagem sem decisão registrada.
- Cliente e servidor usam `game_core` diretamente; nenhuma ponte WASM.
- `game_core` e `truck_sim` não fazem I/O (sem arquivos, rede ou relógio do sistema).
- Conteúdo (modelos, texturas, sons, fontes) só próprio ou com licença que
  permita o uso; registrar origem e licença. Nada de assets de jogos
  comerciais nem de mods derivados deles, nem para teste.
- Ao expandir o mapa, considerar altimetria regional e perfis das rodovias:
  preservar serras, vales e rampas íngremes características. Não aplicar um
  teto genérico de 5,5–7% a todo o mundo. Seguir a política de relevo e escala
  em `docs/MAPA_SUL.md`; altitudes reais ainda não foram importadas.
- Limites explícitos em toda entrada de rede/arquivo; recusa tipada.
- Roadmap atualizado com o que foi entregue, como foi testado e o que falta.
  Não marcar como testado por pessoas o que só passou em teste automatizado.
- Repositório: `https://github.com/rodolfoaltenrath/truck-game-lowpoly`
  (nome provisório). Em 09/11/2026 o proprietário autorizou commit e envio
  (push) ao concluir blocos validados, **sempre sem coautor** (nenhum trailer
  `Co-authored-by`). Não inferir autorização para releases ou mensagens
  externas.

## Ambiente

- Rust via rustup, toolchain fixado em `rust-toolchain.toml`.
- Windows: MSVC ou GNU/MinGW; configuração local em `docs/WINDOWS_DEV.md`.
- Linux: compilador C, ALSA, OpenGL e bibliotecas de desenvolvimento X11/Wayland.
- WSL não substitui validação numa instalação Linux com GPU real.
- Push para `origin` usando a autenticação já configurada.

## Fluxo de desenvolvimento

```
cargo run -p truck_server        # ws://0.0.0.0:39100
cargo run -p truck_client -- --solo
cargo fmt --all --check
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
```

Use `--smoke-frames N --report caminho.json --data-dir pasta` para provas
nativas isoladas. A pose e as preferências ficam em arquivo local limitado.
`ROADMAP_JOGO_WEB.md` registra o histórico anterior à migração; testes web
antigos não provam o comportamento do cliente nativo.
