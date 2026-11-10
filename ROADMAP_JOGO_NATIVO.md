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

## Relevo físico e escala regional

- Proprietário definiu escala **1:20 entre cidades**. Caminhão, pista e pátios
  permanecem em 1:1; atlas atual continua esquemático. Antes de construir toda
  a rede nessa escala, ampliar limites de coordenadas/protocolo e implementar
  carregamento por regiões. As 32 cidades ainda não são todas dirigíveis.
- `game_core::terrain`: trecho piloto com curvas em S para os dois lados,
  elevação de 14 m e inclinação abaixo de 7%, amostrado em nós de 8 m. Pátios
  e acessos de carga mantidos planos. Perfil próprio, sem dados geográficos externos.
- Visual, pintura, cenário e colisão compartilham o mesmo perfil interpolado.
  Colisão acrescenta apenas 160 triângulos. Mapa local acompanha as curvas reais.
  Poses antigas abaixo do novo terreno são elevadas na restauração; poses válidas
  acima da superfície são preservadas.
- Windows GNU: **58 testes do workspace passaram** e Clippy com todos os alvos
  passou. Cobrem altura/curvas/inclinação, continuidade dos pátios, correspondência
  do desenho com o perfil físico, orçamento e orientação das faces, e seis rodas
  apoiadas em subida, crista e descida. Teste manual de condução ainda pendente.
- Smoke gráfico na Intel HD 4000: caminhão sobre a crista permaneceu a ~15,08 m
  de altura (terreno de 14 m), com seis rodas apoiadas e baú engatado; captura do
  circuito e do mapa local curvo inspecionadas. Não é benchmark comparativo nem
  validação humana de uma viagem completa. Pacote portátil não regenerado.

## Atlas regional e primeiro circuito de frete

- Rede própria em `game_core::south`: 32 cidades (11 PR, 11 SC, 10 RS),
  47 ligações bidirecionais e 12 áreas de apoio propostas. Busca de rotas curta,
  rápida e sem pedágio, com simulação de um trecho fechado e recusas tipadas.
- Atlas vetorial nativo: seleção de destino/origem, preferências e destaque do
  caminho. Coordenadas e distâncias esquemáticas, pedágios fictícios/provisórios;
  os corredores regionais ainda não são estradas 3D.
- Primeira versão do circuito era plana: pátios de União da Vitória e Porto União separados por
  800 m, ligação direta e circuito alternativo. Mapa local com posição do jogador.
  Novas partidas começam no pátio; posições salvas continuam preservadas. R volta
  ao pátio e reinicia o treino. Prédios, bombas e relevo são decorativos sem colisões.
- Vegetação limitada por proximidade a no máximo 11 lotes de 200 m, além do
  descarte pelo frustum; morros agrupados em um horizonte separado. A seleção
  muda por célula, sem reconstruir malhas. Isso ainda não é streaming de memória:
  os lotes permanecem residentes. Não há benchmark comparativo de FPS.
- Primeiro frete local: aceitar, carregar, dirigir, descarregar e preparar retorno.
  Carga/descarga leva 2 s com cavalo e baú engatados e parados na área de 48 × 48 m;
  movimento, saída da área ou desengate cancela. Sem dinheiro, carreira, persistência
  do treino, sincronização de fretes ou vagas reservadas pelo servidor.
- Validação Windows GNU: suíte workspace com **55 testes passou**, incluindo
  conectividade entre todos os pares de cidades, alternativas sem pedágio,
  desvio da travessia inicial, ciclo do frete, cancelamentos, entradas inválidas,
  mosaico/área/orçamento da estrada, folga dos seis pneus e rede WebSocket real.
  Clippy do workspace com todos os alvos passou. Builds incrementais do cliente
  reaproveitaram a engine (aproximadamente 15–20 s nas execuções observadas).
- Smokes gráficos Windows/OpenGL 3.3 na Intel HD 4000: mapas local e regional
  (90 quadros cada) e circuito (120 quadros) concluíram com seis contatos de roda.
  Capturas inspecionadas; marcador dinâmico, centralização e caracteres corrigidos.
  Essas execuções curtas não são benchmark. Direção e entrega completas com pessoa,
  avaliação em Linux e pacote portátil atualizado continuam pendentes.

## Revisão visual e desenvolvimento incremental

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
- Formatação Rust, sintaxe PowerShell, testes e Clippy conferidos na etapa do
  circuito piloto. Nenhuma melhoria de desempenho foi medida.

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
| F4 — mundo | Atlas de 32 cidades e circuito piloto; corredores 3D e streaming pendentes |
| F5 — fretes | Exercício local com carga/entrega; regras autoritativas e economia pendentes |
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
não são dirigíveis. O circuito piloto valida acessos e o ciclo básico; antes de
expandir, testar direção/entregas manualmente, construir colisões e serviços e
implementar reserva de vagas e fretes autoritativos.
