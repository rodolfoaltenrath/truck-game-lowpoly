# Roadmap — Jogo de caminhão 3D multiplayer para navegador

> Documento de planejamento. Nada aqui está implementado ainda, salvo onde
> dito explicitamente. O `ROADMAP.MD` existente continua sendo o histórico do
> mod para 18 Wheels of Steel e **não** descreve este jogo.

---

## 1. Visão

Um jogo **original** de caminhão em 3D que roda direto no navegador, sem
instalação, com:

- direção de caminhão com reboque e física convincente;
- mundo aberto com estradas, cidades e empresas de destino;
- carreira com fretes, economia, frota e competição entre jogadores;
- **multiplayer desde o início**: primeiro em LAN, depois hospedado;
- interface e textos em **PT-BR**, moeda **R$**.

O objetivo **não** é recriar 18 Wheels of Steel nem qualquer outro jogo
existente: mapa, caminhões, marcas, arte, nomes e interface são próprios. O
que se aproveita do mod é **código e experiência de engenharia** (protocolo,
sessão, interpolação, economia), não conteúdo do jogo comercial.

### Pilares

1. **Sentir o caminhão**: peso, inércia, frenagem longa, reboque articulado.
2. **Estrada compartilhada**: ver os outros jogadores com movimento suave,
   mesmo com rede ruim.
3. **Economia justa**: o servidor é a autoridade sobre dinheiro, contratos e
   pagamentos. Nada é pago duas vezes, nada some.
4. **Carregar rápido**: jogável em menos de ~30 s numa conexão doméstica
   comum. O mundo restante é baixado durante o jogo.

---

## 2. Princípios de engenharia

| Princípio | Consequência prática |
|---|---|
| Rust para tudo que precisa ser **confiável** | Servidor, protocolo, economia, validação, parsers. |
| TypeScript para tudo que é **apresentação** | Render, câmera, input, áudio, HUD, menus. |
| **Uma** implementação por regra | Lógica compartilhada fica em Rust e roda no cliente via WASM; nada de reescrever em TS. |
| Núcleo **sem I/O** (sans-IO) | `game_core` recebe e devolve bytes/eventos; sockets e relógio ficam por fora. Testável sem rede. |
| Servidor autoritativo onde importa | Dinheiro, contratos, posse, sessão. A posição do próprio caminhão é do cliente, validada pelo servidor (seção 7). |
| Limites explícitos | Todo buffer, fila, mensagem e arquivo tem tamanho máximo e recusa tipada, como no mod. |
| Testar sem humano primeiro | Simulação, fixtures, Playwright, soak. O teste manual fica agrupado por marco. |
| Não declarar pronto o que não foi provado | O roadmap separa "entregue", "testado na bancada" e "testado por pessoas". |

---

## 3. Stack

### Linguagens

| Uso | Escolha |
|---|---|
| Servidor, núcleo compartilhado, ferramentas | **Rust** (edição 2024, toolchain fixado em `rust-toolchain.toml`) |
| Cliente web | **TypeScript** (modo `strict`) |
| Shaders | **WGSL/GLSL**, de preferência via Node Material do Babylon |
| UI | **HTML + CSS** sobre o canvas |
| Persistência | **SQL** (SQLite; Postgres só se a hospedagem exigir) |

Fora de escopo: Python, C#, Go e linguagens de script para mods, até decisão
explícita em contrário.

### Bibliotecas e ferramentas

| Área | Escolha | Observação |
|---|---|---|
| Motor 3D | **Babylon.js** | WebGPU com fallback WebGL2; glTF, PBR, sombras e inspector prontos. |
| Física | **Rapier** (`@dimforge/rapier3d` no cliente, crate `rapier3d` no servidor) | Mesmo motor dos dois lados. Controlador de veículo por raycast; reboque por junta. |
| Ponte Rust↔TS | **wasm-bindgen** + **wasm-pack** | Tipos gerados para TS com **ts-rs** ou **specta**. |
| Build web | **Vite** + **pnpm** | Recarga rápida no desenvolvimento. |
| Lint/format TS | **Biome** | Um só programa para as duas coisas. |
| Testes TS | **Vitest** (unidade) + **Playwright** (ponta a ponta em navegador real) | |
| Testes Rust | `cargo test`, `network_lab`, fuzz e soak herdados do mod | |
| Servidor | **Tokio** + **tokio-tungstenite** (WebSocket) + **wtransport** (WebTransport) | |
| Banco | **SQLite** via `sqlx` ou `rusqlite` | Migrações versionadas no repositório. |
| Arte | **Blender** → glTF/GLB | |
| Otimização de assets | **gltf-transform** (meshopt/Draco, texturas KTX2) | Roda no pipeline de build, não no jogo. |
| HTTPS local/produção | **Caddy** | Exigido pelo WebTransport. |
| CI | **GitHub Actions** | Rust nativo, WASM, TS, Playwright headless. |

---

## 4. Arquitetura

```
┌──────────────────────── Navegador ────────────────────────┐
│                                                            │
│  web/ (TypeScript)                                         │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ ┌────────────────┐  │
│  │ Render   │ │ Input /  │ │ UI DOM  │ │ Rede (WS / WT) │  │
│  │ Babylon  │ │ câmera   │ │ HUD/menu│ │ só bytes       │  │
│  └────▲─────┘ └────┬─────┘ └────▲────┘ └──────┬─────▲───┘  │
│       │            │            │             │     │      │
│  ┌────┴────────────▼────────────┴─────────────▼─────┴───┐  │
│  │ game_core.wasm (Rust, sans-IO)                        │  │
│  │ sessão · eventos confiáveis · interpolação · regras   │  │
│  └───────────────────────▲───────────────────────────────┘  │
│                          │                                  │
│  ┌───────────────────────┴───────┐                          │
│  │ Rapier (WASM) — física local  │                          │
│  └───────────────────────────────┘                          │
└──────────────────────────┬─────────────────────────────────┘
                           │  WebTransport (datagramas + streams)
                           │  WebSocket (fallback)
┌──────────────────────────▼─────────────────────────────────┐
│ server/ (Rust, Tokio)                                       │
│  transporte → game_core (mesmo crate) → salas/sessões       │
│  economia autoritativa · validação de movimento · trânsito  │
│  SQLite (contas, empresas, frota, ledger, contratos)        │
│  HTTP estático: cliente web + assets por região             │
└─────────────────────────────────────────────────────────────┘
```

### Responsabilidades

- **Cliente TS**: desenha, lê o controle, roda a física do **próprio**
  caminhão, envia poses e pedidos ("aceitar frete", "entregar") e mostra o
  estado que o servidor confirma.
- **game_core (Rust → WASM e nativo)**: formato de mensagens, handshake,
  heartbeat, retomada, eventos confiáveis com ACK/dedup, janela de eventos,
  base de tempo, interpolação/extrapolação, validação de limites. O mesmo
  código compila para o cliente e para o servidor.
- **Servidor**: salas, identidade, retransmissão de poses, validação de
  plausibilidade, economia, contratos, persistência, trânsito IA (fase
  posterior), relógio do mundo.

---

## 5. Estrutura do repositório (alvo)

```
Cargo.toml                 workspace Rust
rust-toolchain.toml
crates/
  game_core/               sans-IO: protocolo, sessão, eventos, interpolação, tempo
  game_rules/              economia, contratos, ledger, carreira (puro, sem I/O)
  server/                  binário Tokio: transporte, salas, SQLite, HTTP
  world_tools/             CLI: mapa, estradas, regiões, empacotamento de assets
  game_core_wasm/          bindings wasm-bindgen + geração de tipos TS
web/
  package.json, vite.config.ts, tsconfig.json, biome.json
  src/
    main.ts
    engine/                cena, câmera, luz, céu, streaming de regiões
    vehicle/               caminhão, reboque, rodas, áudio do motor
    physics/               integração Rapier
    net/                   transporte WS/WT, ponte com game_core.wasm
    ui/                    menus, HUD, garagem, mapa, i18n
  tests/                   Vitest
  e2e/                     Playwright
assets-src/                fontes do Blender (Git LFS ou fora do Git; decidir)
assets/                    GLB/KTX2 gerados (não versionados; gerados no build)
docs/
  ARCHITECTURE.md
  NETWORK_PROTOCOL.md      herdado e adaptado do mod
  GAME_DESIGN.md
  VALIDATION.md            roteiros de teste manual agrupado
legacy/mod-18wos/          código do mod, fora do workspace (decisão pendente)
```

---

## 6. Reaproveitamento do mod

| Origem no repo atual | Destino | Ação |
|---|---|---|
| `multiplayer_server/src/session_protocol.rs`, `session_hub.rs`, `session_client.rs` | `game_core` | Extrair. Trocar UDP cru por transporte abstrato. |
| `event_window.rs`, `session_timing.rs`, `identity.rs` | `game_core` | Extrair. A identidade passa a viver em IndexedDB ou no servidor, não em arquivo. |
| `career_world.rs`, `career_store.rs`, `career.rs` | `game_rules` + `server` | Extrair a lógica pura. Trocar o armazenamento por SQLite. |
| `vehicle_appearance.rs`, `appearance_stream.rs` | `game_core` / `game_rules` | Reaproveitar o catálogo, o descritor e o stream de aparência. Os modelos serão os nossos. |
| `world_join.rs` | `server` | Reaproveitar a ideia de snapshot + fila com corte. |
| `client_dll/src/interpolation.rs`, `presentation.rs` | `game_core` | Extrair a interpolação (100 ms / 200 ms, proteção contra rajada). |
| `network_conditions.rs`, `network_lab` | `game_core` (dev) | Manter como bancada offline de rede. |
| `fuzz.rs`, `parser_fuzz.rs`, `soak.rs` | testes | Manter e adaptar aos novos parsers. |
| `mod_ui` (layout/flow) | referência | Ideias de fluxo; a UI nova é DOM. |
| `client_dll` (hooks, memória, PE, gates, sondas) | **descartar** | Existia para contornar motor fechado. |
| `asset_tools` (SCS, l10n do jogo, kit LAN) | **descartar** | Só a disciplina de manifesto/hash vai para `world_tools`. |

**Ajustes obrigatórios para compilar em WASM:**

- `std::time::Instant` → crate `web-time`;
- `getrandom` com a feature `wasm_js`;
- nada de `std::fs`/`std::net` dentro de `game_core`;
- uma thread só no cliente no início (threads WASM exigem COOP/COEP).

---

## 7. Rede (netcode)

### Transporte

| Fase | Canal | Uso |
|---|---|---|
| MVP | **WebSocket** | Tudo. Simples, funciona em qualquer lugar; aceita a latência extra do TCP. |
| Depois | **WebTransport** | Datagramas para poses (sem garantia de entrega). Streams confiáveis para eventos. WebSocket vira fallback. |

### Modelo

- **Caminhão próprio**: simulado localmente (resposta imediata). O cliente
  envia a pose a **20–30 Hz** com timestamp do servidor.
- **Caminhões remotos**: interpolação de snapshot com atraso de **100 ms** e
  extrapolação de até **200 ms**, como no mod. Somem com transição suave
  quando não há pose fresca.
- **Validação no servidor**: velocidade máxima, aceleração, teletransporte e
  saída do mapa. Uma violação gera correção ou recusa; nunca altera economia.
- **Eventos confiáveis**: aceitar frete, acoplar reboque, entregar, pagar. Usam
  stream + id, ACK e dedup (herdado do `event_window`).
- **Colisão entre jogadores**: começa **desligada** (fantasma). Ligar exige
  autoridade de física no servidor, que fica para a fase F11.
- **Orçamento**: ≤ 1200 bytes por datagrama; limite por cliente e por sala;
  rosters revisionados.

### Salas

- Sala = mundo compartilhado. LAN primeiro: o servidor roda em um PC e os
  outros abrem `https://ip:porta`.
- Até 16 jogadores por sala no MVP; medir antes de ampliar (o soak do mod
  aguentou 32–64 em loopback).

---

## 8. Física e veículo

- **Cavalo mecânico**: corpo rígido + 4–6 rodas por raycast (suspensão, atrito
  longitudinal/lateral, freio, freio motor).
- **Trem de força**: curva de torque, câmbio (automático primeiro, manual
  depois), embreagem simplificada, retarder.
- **Reboque**: corpo separado ligado por **junta esférica** na quinta-roda,
  com limites de ângulo. Massa varia com a carga.
- **Passo fixo** de 60 Hz, desacoplado do FPS de render.
- **Ajuste por dados**: cada caminhão e reboque é um arquivo de definição
  (massa, centro de massa, rodas, torque); nada de números fixos no código.
- **Danos** (fase posterior): leves, sem deformação de malha no início; afetam
  pagamento.

---

## 9. Mundo

### Formato do mapa

- Estradas definidas como **splines** com perfil (faixas, acostamento,
  sinalização). `world_tools` gera a malha da estrada e o colisor.
- Terreno em **heightmap por região** (tiles de, por exemplo, 1 km × 1 km).
- Cidades e empresas como **prefabs** posicionados (galpões, docas,
  estacionamento, postos).
- Tudo descrito em arquivos de dados versionados, com manifesto e hash por
  região (mesma disciplina do mod).

### Streaming

- Só a região atual e as vizinhas ficam carregadas.
- Níveis de detalhe (LOD) e impostores para objetos distantes.
- O primeiro download contém só o necessário para a garagem e a região
  inicial.

### Editor

- **Fase inicial**: editar splines/prefabs num arquivo de dados e gerar pela
  CLI.
- **Depois**: um editor no próprio navegador (modo dev do cliente), que salva
  o mesmo formato.

### Ambiente

- Ciclo dia/noite com relógio do servidor (o mesmo para todos na sala).
- Clima simples (chuva, neblina) na fase F8.

---

## 10. Jogabilidade

### Loop básico

1. Ver fretes disponíveis no mapa ou no quadro da empresa.
2. Aceitar um frete (exclusivo: dois jogadores não pegam o mesmo).
3. Ir até a origem, acoplar o reboque.
4. Dirigir até o destino respeitando prazo; combustível e danos contam.
5. Estacionar na doca, entregar e receber pagamento (único, confirmado pelo
   servidor).

### Carreira

- Cada jogador tem uma **empresa** com saldo, caminhões e reputação.
- Ao criar a sala/carreira: **meta financeira** ou **Sem limite**. Sem limite
  desliga só a condição de vitória; economia e competição continuam.
- Evolução: comprar caminhões, contratar motoristas IA (fase F10), ranking.

### Economia (autoritativa no servidor)

- Ledger com lançamentos imutáveis; saldo é derivado do ledger.
- Pagamento de contrato com **deduplicação persistente** (herdado do
  `career_world`): repetir o pedido não paga de novo.
- Custos: combustível, pedágio, manutenção, multas (opcional).
- Valores em **R$** com balanceamento próprio (planilha de referência em
  `docs/GAME_DESIGN.md`).

---

## 11. Interface e localização

- Telas: entrada (nome, sala), garagem, mapa/fretes, HUD de direção
  (velocímetro, marcha, combustível, GPS, prazo), pausa, configurações.
- **PT-BR primeiro**, com chaves de texto em arquivo próprio para permitir
  outros idiomas depois.
- Formatação de moeda e números com `Intl` (R$ 1.234,56).
- Controles: teclado, mouse, **gamepad** (Gamepad API); volante é desejável
  depois.
- Acessibilidade mínima: legendas, remapeamento de teclas, escala da UI.

---

## 12. Áudio

- Motor com camadas por RPM e carga, ar do freio, seta, buzina, ambiente.
- Web Audio API pelo Babylon; volume por categoria.
- Sons próprios ou com licença compatível, registrados com origem.

---

## 13. Persistência

| Dado | Onde |
|---|---|
| Identidade local (antes de contas) | IndexedDB no navegador + chave no servidor |
| Empresas, frota, ledger, contratos | SQLite no servidor |
| Configurações do jogador | `localStorage` (gráficos, controles) |
| Snapshot da sala (relógio, fretes abertos) | SQLite, com checkpoint periódico |

- Migrações versionadas; backup antes de migrar.
- Contas reais (login) só na fase de hospedagem pública.

---

## 14. Metas de desempenho

| Métrica | Meta |
|---|---|
| FPS | 60 num notebook com GPU integrada recente, preset "médio" |
| Download inicial | ≤ 15 MB até a garagem (WASM + JS + região inicial, comprimidos) |
| Tempo até dirigir | ≤ 30 s em 50 Mbit/s |
| Memória | ≤ 1,5 GB de aba no preset médio |
| Rede por jogador | ≤ 10 KB/s de subida em direção normal |
| Latência percebida dos remotos | atraso de interpolação ≤ 150 ms |

Presets gráficos: baixo/médio/alto (sombras, distância, LOD, resolução de
render).

---

## 15. Testes e validação

| Camada | Ferramenta | O que prova |
|---|---|---|
| Lógica Rust | `cargo test` | Protocolo, eventos, economia, limites. |
| Rede simulada | `network_lab` (herdado) | Perda, jitter, reordenação, blackout, sem navegador. |
| Robustez | fuzz de parsers | Nenhum panic com entrada adulterada. |
| Carga | soak (herdado) | N clientes, quedas e retomadas, servidor sem vazamento de sessão. |
| TS | Vitest | Funções puras de UI, conversões, input. |
| Ponta a ponta | Playwright (Chromium/Firefox/WebKit) | Duas abas se veem; aceitar e entregar frete; reconexão. |
| Desempenho | captura no Playwright + painel de FPS | Metas da seção 14 por preset. |
| Manual | roteiro em `docs/VALIDATION.md` | Sensação de direção, suavidade em LAN real, dois PCs. |

Regra herdada: **nunca** marcar como testado por pessoas o que só passou
em fixture.

---

## 16. Fases

Cada fase lista as entregas e os critérios de conclusão. Uma fase só é
fechada quando os critérios forem provados.

### F0 — Fundação
- Repositório git inicializado; código do mod movido para `legacy/` (ou
  removido do workspace).
- Novo `AGENTS.md` com as regras deste jogo (Rust + TS, sem outras
  linguagens; seção 18).
- Workspace Rust com `game_core`, `server`, `game_core_wasm` vazios.
- `web/` com Vite + TS + Babylon renderizando uma cena vazia.
- CI: `cargo test`, build WASM, `pnpm build`, Biome.

**Pronto quando:** `cargo test` e `pnpm build` passam na CI e a página abre
uma cena 3D vazia.

### F1 — Protótipo de direção
- Caminhão de teste (caixa + rodas) dirigível num plano com rampas.
- Rapier com passo fixo; câmera de perseguição e de cabine.
- Teclado e gamepad.

**Pronto quando:** dá para acelerar, frear, fazer curva e dar ré com
comportamento estável a 60 FPS; teste Playwright abre a página e confirma que
o veículo se move ao receber input simulado.

### F2 — Núcleo compartilhado e rede mínima
- Extrair `game_core` do `multiplayer_server` (seção 6), compilando nativo e
  WASM.
- Servidor com WebSocket, sala única, handshake, heartbeat e retransmissão de
  poses.
- Cliente envia e recebe poses e interpola os remotos.
- `network_lab` portado para o novo núcleo.

**Pronto quando:** duas abas veem o caminhão uma da outra, suave, inclusive
com perda/jitter simulados; teste Playwright de duas abas na CI.

### F3 — Caminhão e reboque de verdade
- Primeiro modelo próprio de cavalo mecânico e de reboque (Blender → GLB).
- Junta da quinta-roda, acoplar/desacoplar, massa da carga.
- Câmbio automático, curva de torque, freio motor.
- HUD básico (velocidade, marcha, RPM).

**Pronto quando:** reboque segue de forma crível em curvas e em ré;
acoplamento sincroniza para os outros jogadores.

### F4 — Primeiro mundo
- Formato de mapa (splines + heightmap + prefabs) e `world_tools` gerando
  malhas e colisores.
- Uma região pequena: duas cidades, uma rodovia entre elas, um posto.
- Streaming por região e LOD.

**Pronto quando:** viagem completa entre as duas cidades sem travadas de
carregamento, dentro das metas da seção 14.

### F5 — Fretes (single-player primeiro)
- Empresas de origem/destino, catálogo de cargas, prazo, pagamento.
- Combustível e postos.
- Tela de fretes e GPS com rota.

**Pronto quando:** loop completo aceitar → acoplar → entregar → receber
funciona do início ao fim.

### F6 — Multiplayer de verdade
- Fretes exclusivos e pagamento autoritativo no servidor (`game_rules`).
- Eventos confiáveis para acoplar/entregar.
- Validação de plausibilidade de movimento.
- WebTransport com fallback WebSocket.
- Reconexão sem perder o frete em andamento.

**Pronto quando:** dois PCs na LAN completam fretes ao mesmo tempo; um frete
não pode ser pego por dois; derrubar a rede no meio da entrega não paga duas
vezes nem perde o pagamento. Testado por pessoas (roteiro em VALIDATION.md).

### F7 — Carreira e persistência
- SQLite com empresas, frota, ledger e contratos; migrações.
- Meta financeira ou Sem limite na criação da carreira.
- Comprar caminhões; garagem.
- Retomar a carreira após reiniciar o servidor.

**Pronto quando:** reinício do servidor preserva saldos e fretes; ranking da
sala é consistente com o ledger.

### F8 — Ambiente
- Ciclo dia/noite sincronizado, faróis, iluminação de cidade.
- Clima (chuva, neblina) com efeito na aderência.

**Pronto quando:** todos os jogadores da sala veem a mesma hora e clima.

### F9 — UI completa, PT-BR, áudio
- Todas as telas da seção 11, textos em PT-BR, R$.
- Áudio da seção 12.
- Configurações gráficas e de controle salvas.

**Pronto quando:** um jogador novo entra, configura e completa um frete sem
ajuda externa.

### F10 — Trânsito IA e frota IA
- Carros e caminhões IA nas rodovias, controlados pelo servidor e replicados.
- Motoristas IA da empresa que fazem fretes "em segundo plano" (simulação
  econômica, não física).

**Pronto quando:** trânsito é igual para todos os jogadores da região, sem
custo de rede acima da meta.

### F11 — Física autoritativa e colisão entre jogadores (opcional)
- Rapier no servidor para regiões com jogadores próximos.
- Colisão entre jogadores com correção de estado.

**Pronto quando:** colisões são estáveis com 100 ms de latência; caso
contrário, manter o modo fantasma.

### F12 — Hospedagem pública
- Contas (login), HTTPS com certificado real, limites por IP.
- Deploy do servidor (binário único + Caddy + SQLite/Postgres).
- Observabilidade: logs estruturados, métricas de sala.

**Pronto quando:** jogadores fora da LAN jogam juntos por 1 h sem queda do
servidor.

### F13 — Polimento e lançamento
- Mais caminhões, reboques, regiões.
- Tutorial, conquistas, balanceamento.
- Página do jogo, política de privacidade, créditos e licenças.

---

## 17. Riscos

| Risco | Mitigação |
|---|---|
| Física de reboque instável em WASM | Prototipar cedo (F1/F3); passo fixo; ajuste por dados. |
| Desempenho em GPU integrada | Presets desde o início; medir em cada fase. |
| Tamanho do download | Streaming por região, KTX2/meshopt, orçamento na CI. |
| Suporte a WebTransport/WebGPU por navegador | Fallbacks WebSocket/WebGL2 obrigatórios. |
| Produção de arte (é o maior volume de trabalho) | Começar com formas simples; padronizar o pipeline Blender cedo; usar assets CC0 com origem registrada. |
| Trapaça em economia | Servidor autoritativo + ledger imutável desde F6. |
| Escopo grande | Fechar fases em ordem; nada de F10+ antes do loop F5/F6 estar sólido. |

---

## 18. Regras do projeto (base para o novo AGENTS.md)

- Rust para servidor, núcleo compartilhado, regras e ferramentas.
- TypeScript só no cliente web. Shaders, HTML/CSS e SQL quando necessários.
- Nenhuma outra linguagem sem decisão registrada.
- Lógica compartilhada não é reescrita em TS: passa por `game_core.wasm`.
- Todo conteúdo (modelos, texturas, sons, fontes) tem origem e licença
  registradas; nada de assets de jogos comerciais.
- Limites explícitos em toda entrada de rede/arquivo; recusa tipada.
- Commits por bloco validado; roadmap atualizado com o que foi entregue, como
  foi testado e o que falta.

---

## 19. Decisões pendentes

1. **Mapa**: país/região fictícia ou inspirada em região real (por exemplo,
   rodovias brasileiras estilizadas)?
2. **Código do mod**: mover para `legacy/` neste repositório ou manter só no
   repositório antigo?
3. **Assets fonte**: Git LFS ou armazenamento separado?
4. **Colisão entre jogadores**: aceitar o modo fantasma como definitivo ou
   investir na F11?
5. **Nome do jogo** e identidade visual.
6. **Mobile**: suportar celular/tablet (controles touch) ou só desktop?
7. **Licença** do código do jogo (o mod usa Apache-2.0).

---

## 20. Estado atual

### 09/11/2026 — F0 parcial, F1 entregue na bancada

- Código do mod movido para `legacy/mod-18wos/` (pendência 2 resolvida assim;
  fora do workspace e **fora do Git**: já está no repositório antigo do mod).
  Novo `AGENTS.md`/`README.md`.
- Repositório criado pelo proprietário: `rodolfoaltenrath/truck-game-lowpoly`
  (nome provisório). Commits e push autorizados por bloco validado, sem
  coautor.
- Workspace Rust com `crates/game_core` (placeholder). `cargo check` e build
  `wasm32-unknown-unknown` passam. Testes nativos Rust **não rodam** até
  instalar `gcc` (sem sudo nesta sessão).
- `web/`: Vite + TS strict + Babylon 9 + Rapier 0.21 + Biome + Playwright.
  `pnpm check` passa.
- F1: cavalo mecânico de formas simples, 6 rodas (raycast), tração traseira
  dupla, câmbio automático de 8 marchas + ré arcade, arrasto/rolamento,
  freio de mão, luzes de freio. Pista procedural sem assets externos: oval de
  asfalto, rampas, slalom de cones, pilha de caixas, área de morros (trimesh).
  Câmeras perseguição/cabine/livre, teclado e gamepad, HUD com velocidade,
  marcha e rpm.
- Teste Playwright (`e2e/drive.spec.ts`) passou: assenta nas 6 rodas, passa de
  30 km/h com troca de marcha, esterça para o lado certo, freia até parar e
  engata ré, sem erros no console.
- Ajustes após o primeiro teste do proprietário ("ficou ótimo", mas
  "travado"): rodas com aro/raios/sulcos visíveis girando; mouse orbita a
  câmera (perseguição e cabine) e roda dá zoom; desenho interpolado entre
  passos de física; pedais com rampa; troca de marcha sem cortar a tração;
  peças do caminhão e tracejado da pista fundidos (menos chamadas de desenho);
  estáticos congelados; sombra PCF numa área fixa em volta do caminhão.
  Teste `e2e/smooth.spec.ts`: variação da velocidade aparente caiu de 0,35
  para 0,16–0,21 no Chromium headless (render por software; limite 0,25).
- Direção de arte definida pelo proprietário: **low poly charmoso, não
  realista** (guia em `docs/ART_STYLE.md`). Primeira passada: caminhão de
  cabine avançada chanfrado com faróis/lanternas acesos e brilho, paleta
  única, capim dourado facetado, céu em degradê, sol, nuvens, ~700 árvores e
  pedras (thin instances com colisor), pós-processamento por qualidade
  (G) e troca de pintura (P). O teste de suavidade agora compara com e sem
  interpolação no mesmo trecho: variação ~0,39 → ~0,20 em três execuções.
- **Pendente**: sensação de direção e suavidade confirmadas por pessoa; FPS em GPU real; CI;
  `gcc` para testes Rust nativos.

### 09/11/2026 — F3 (reboque) primeira entrega, na bancada

- Rodas por raycast, modelo e interpolação extraídos para `vehicle/wheels.ts`
  e compartilhados por cavalo e reboque.
- Três semirreboques low poly por dados (`vehicle/trailer.ts`): baú 12 m
  (15,5 t), graneleiro 11 m (22 t, caçamba com carga), toras 12 m (19,5 t,
  fueiros e toras com pontas claras). Três eixos, pés de apoio, protetores
  laterais, faixas refletivas, para-choque listrado e lanternas.
- Engate (`vehicle/hitch.ts`): junta esférica do Rapier entre o prato da
  quinta-roda e o pino-rei. Engata sozinho ao encostar de ré (como a
  mandíbula real), dentro de 1 m na horizontal, −0,45..+0,35 m na vertical,
  35° de alinhamento e até 9 km/h; **T** desengata parado. Reboque solto fica
  com freio de estacionamento e apoiado nos pés; engatado, freia junto com o
  cavalo e acende as lanternas.
- Ré limitada a 8 km/h. Câmera de perseguição se afasta com reboque. HUD
  mostra reboque e massa; mensagens de engate.
- Começa com o baú engatado; graneleiro e toras estacionados ao lado da área
  de testes.
- Teste `e2e/trailer.spec.ts` passou: reboque acompanha a curva com atraso e
  folga da junta < 0,2 m, desengata, fica parado com o cavalo se afastando e
  volta a engatar sozinho dando ré em linha reta. Suíte completa: 5 testes.
- **Limitações**: cavalo e reboque não colidem entre si (grupos de colisão),
  então um L fechado ou ré desalinhada atravessa; junta esférica permite
  rolagem livre; carga é massa fixa; sem tombamento de carga.
- **Pendente**: sensação de dirigir com reboque testada por pessoa.

### Próximo passo

Ajustar direção e reboque com o retorno do proprietário no navegador; depois
F2 (extrair `game_core` e ligar dois navegadores pelo servidor, já com o
reboque sincronizado).
