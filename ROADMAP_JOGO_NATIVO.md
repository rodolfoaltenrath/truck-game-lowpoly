# Roadmap — jogo de caminhão nativo em Rust

## Decisão de 10/11/2026

O proprietário autorizou migrar o projeto para **100% Rust**, com executáveis
para **Windows e Linux**, sem navegador embarcado. O protótipo web anterior
está documentado em [ROADMAP_JOGO_WEB.md](ROADMAP_JOGO_WEB.md); seus testes e
marcos são históricos, não certificações do novo cliente.

## Stack e divisão

| Parte | Implementação |
|---|---|
| Cliente e interface | Rust, Fyrox 1.0.1, janela nativa, OpenGL 3.3 |
| Física | `truck_sim`, Rapier 0.32, passo fixo de 60 Hz, sem I/O |
| Regras e protocolo | `game_core`, compartilhado diretamente, sem I/O |
| Sala multiplayer | `truck_server`, Rust/Tokio, WebSocket LAN |
| Conteúdo | Geometria procedural própria, low poly, paleta original |
| Distribuição | Pasta portátil por sistema, cliente e servidor separados |

WebSocket é somente o transporte de rede; não requer navegador. OpenGL foi
escolhido para alcançar também hardware antigo. Isso não garante ganho de
FPS: desempenho e compatibilidade precisam ser medidos no executável.

## Revisão visual — validação executável adiada

Ciclo rápido acrescentado: `scripts/dev.ps1` executa somente o cliente no perfil
dev, reaproveitando o cache existente; `-Check` verifica cliente e testes sem gerar
executável nem rodar testes. Mantém a preparação GNU do script completo, sem mudar
perfis ou flags para forçar reconstrução. Sintaxe PowerShell conferida; nenhum
build ou teste executado nesta alteração, conforme a solicitação de adiamento.

O proprietário compilou e confirmou que o cliente abriu e funcionou nesta revisão.
Isso não certifica desempenho, qualidade visual ou a suíte automatizada completa.
O pacote em `dist/` ainda corresponde à base anterior.

- Cabine com defletor, pala, frisos, grade e detalhes de portas; baú com faixas,
  nervuras e ferragens. Vidros com tonalidade mais definida.
- Copas facetadas, pinheiros, pedras, galpões e morros decorativos originais.
  Vegetação agrupada em trechos de 200 m para descarte pelo frustum, com materiais
  compartilhados entre lotes. O relevo é visual, sem colisões novas.
- Solo, acostamento, asfalto e pintura formam mosaico horizontal sem sobreposição.
  A rodovia usa 4.020 triângulos; remove caixas chanfradas das marcações. O cenário
  ganhou geometria, portanto a redução da pista não comprova ganho global de FPS.
- Novo teste verifica plano, orientação, área total e orçamento da rodovia.
  O teste existente de folga dos pneus permanece e deve ser reexecutado.
- `scripts/build-native.ps1` prepara GNU local quando aplicável, usa dois trabalhos
  de compilação por padrão, verifica formato, executa testes e Clippy e gera pacote.
  `-Smoke` executa 600 quadros com relatório e captura; `-Dev` usa desenvolvimento.
- Verificações desta revisão: formatação Rust, análise sintática PowerShell e
  `git diff --check`. Testes, compilação, inspeção gráfica e medição de FPS ficam
  pendentes até executar o script. Nenhuma melhoria de desempenho foi medida.

## Primeira base nativa

- Caminhão de cabine avançada, seis rodas raycast, direção, câmbio automático,
  frenagem, ré, freio de mão e reboque baú articulado.
- Engate/desengate manual com T quando parado e próximo do pino.
- Rodovia procedural de validação, câmera externa/frontal/aérea, pintura e HUD.
  A câmera frontal ainda não possui interior de cabine ou painel 3D.
- Preset leve inicial, resolução da janela preservada, SSAO e HDR desligados;
  G habilita sombras. Sem redução automática da resolução.
- Posição, pintura, câmera e sombras salvas fora da instalação; `--data-dir`
  permite manter os dados junto à pasta portátil. Leitura limitada a 4 KiB,
  validação de versão, coordenadas e quaternion, escrita por arquivo temporário.
- Rede nativa com filas limitadas, limites de quadros e handshake, reconexão,
  sessão e interpolação existentes, caminhões e baús remotos.

### Validação desta migração

Primeira base validada em bancada. Não houve ainda validação manual pelo
proprietário ou sessão LAN entre pessoas no cliente nativo.

- Compilação (`cargo check`) e Clippy com todos os alvos no Fedora/WSL passaram.
- Windows GNU: build do cliente e servidor, `cargo test --workspace` (48 testes)
  e Clippy com todos os alvos passaram. Inclui duas sessões nativas por
  WebSocket real, saída explícita, limites de dados locais e teste geométrico
  dos seis pneus em repouso/compressão máxima, também com esterço dianteiro.
- Fedora/WSL: 43 testes de núcleo, física e servidor passaram. Os testes do
  cliente foram compilados pelo Clippy; sua execução gráfica Linux ainda
  está pendente. WSL não equivale a ensaio numa instalação Linux com GPU real.
- Janela Windows executada diretamente na **Intel HD Graphics 4000**, OpenGL
  **3.3.0**, driver **10.18.10.4252**, em **1100 × 700**, sem navegador ou
  SwiftShader. Capturas da janela inspecionadas. Nos smokes curtos, cerca de
  27–37 FPS; isso não é benchmark comparável com a câmera/driver do protótipo
  web e não prova conforto ao dirigir ou estabilidade em sessões longas.
- Smoke em movimento: 600 quadros, 22,9 s, caminhão avançou ~269 m com baú,
  câmbio chegando à sexta marcha, seis contatos de roda e sessão conectada
  com um remoto de eco. Inspecionadas também câmera frontal e sombras.
- Pacote Windows GNU de desenvolvimento gerado e executado fora de `target/`:
  cliente ~105,5 MiB, servidor ~7 MiB, ZIP ~28 MiB, símbolos de depuração
  removidos da cópia distribuída. Importações PE verificadas: somente DLLs
  do Windows, sem navegador ou runtime MinGW externo. Smoke do pacote passou.
- Cliente TypeScript, Vite, testes Playwright e crate de ponte WASM removidos
  do projeto versionado; o histórico completo permanece no commit 346aced.
- Builds e pacotes automatizados configurados para Windows MSVC e Linux,
  incluindo avisos das dependências e runtime Rust. Execuções do CI ainda
  precisam ser conferidas após o push; não confundir configuração com aprovação.

### Diferenças e pendências do port

O cliente nativo começa com o baú e uma rodovia plana de teste. Graneleiro,
toras, engate automático, gamepad, áudio, cenário completo da antiga pista,
trânsito de demonstração e streaming visual por regiões precisam de port e
validação próprios. A seleção de regiões continua disponível em `game_core`;
seu módulo `traffic` mede tráfego de rede, não implementa veículos de IA.

## Fases

| Fase | Estado no cliente nativo |
|---|---|
| F0 — infraestrutura | Base nativa e pacote Windows validados; pacote Linux em CI |
| F1 — direção | Testes e smoke passaram; ajustar suspensão/câmera com pessoas |
| F2 — multiplayer | Transporte e eco validados; sessão LAN com pessoas pendente |
| F3 — reboques | Baú validado em bancada; outros tipos e engate automático pendentes |
| F4 — mundo | Rodovia de teste; mapa do Sul e streaming ainda pendentes |
| F5 — fretes | Pendente |
| F6 — economia | Pendente, servidor autoritativo |
| F7 — carreira | Pendente |
| F8 — ambiente | Pendente |
| F9 — interface e áudio | HUD inicial; menus e áudio pendentes |
| F10 — trânsito | Demonstração anterior em TS; port de veículos e IA pendente |
| F11 — física autoritativa | Avaliar depois do primeiro ciclo de fretes |
| F12 — hospedagem | LAN primeiro; hospedagem pública pendente |
| F13 — lançamento | Pendente; nenhuma release autorizada nesta entrega |

## Próximo conteúdo

Preservar o projeto [MAPA_SUL.md](docs/MAPA_SUL.md): PR, SC e RS, distribuição
geográfica aproximada, distâncias comprimidas, rotas alternativas e serviços
revisados. União da Vitória/Porto União será o polo inicial compartilhado,
com vagas distintas reservadas pelo servidor. As 32 cidades planejadas ainda
não são dirigíveis. Primeiro estabilizar o port nativo, depois construir esse
polo e um circuito pequeno antes de expandir o mapa.
