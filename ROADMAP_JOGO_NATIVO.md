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

## Direção suave e estabilidade do conjunto carregado

- Tombamento reproduzido em curva contínua a 60 km/h. Novo teste verifica
  orientação dos dois corpos; o estresse anterior só garantia poses finitas.
- Esterço assistido conforme a velocidade, transições suaves de direção e
  pedais, amortecimento recalibrado e barras estabilizadoras por eixo.
  Carga distribuída mais baixa, preservando as massas totais do cavalo/baú.
- Regressão cobre curvas a 30/60 km/h e slalom a 80 km/h, mantendo velocidade,
  mudança real de rumo, engate e vertical dos corpos acima de 0,9. Testes da
  serra continuam exigindo apoio das doze rodas em oito pontos nos dois sentidos.
- Detalhes e limites em [DRIVING.md](docs/DRIVING.md). São parâmetros de
  jogabilidade; sensação humana e calibração por modelo real seguem pendentes.
- Windows GNU: 67 testes do workspace, formatação e Clippy passaram; build
  incremental do cliente concluído. Smoke de 600 quadros com aceleração,
  curvas, freio e ré na Intel HD 4000 encerrou com código 0, seis contatos do
  cavalo e baú engatado; captura inspecionada. Janela anunciada e dados isolados.
  Sem benchmark, novo pacote portátil ou nova validação gráfica Linux.

## Diretriz de altimetria para a expansão

- Pedido do proprietário em 10/11/2026: preservar o relevo característico dos
  lugares, com serras e rampas fortes onde cabíveis. Não usar os limites suaves
  dos protótipos como teto global. Política permanente em
  [MAPA_SUL.md](docs/MAPA_SUL.md), também referenciada nas instruções dos agentes.
- Consultar perfis rodoviários e referências altimétricas, adaptar desníveis à
  compressão 1:20 e validar o conjunto carregado na subida e descida. Dados reais
  ainda não importados; os relevos atuais e a serra de 5,5% continuam procedurais.
- Esta etapa altera somente documentação; revisão por diff, sem build ou
  testes de código. Importação de dados e revisão das rampas seguem pendentes.

## Serra do Rio do Rastro — trecho inspirado

- Primeiro desvio dirigível original, com aproximadamente 4,8 km, cinco
  cotovelos de 180°, cerca de 250 m de subida e mirante alargado para retornar.
  Inclinação central até 5,5%, pista com acostamentos, sinalização, barreiras
  de concreto com colisão, encostas facetadas e vegetação em lotes existentes.
- Acesso provisório a 1,4 km ao sul do polo, 600 m após Porto União, pelo lado
  leste. M mostra o traçado e R retorna ao polo. Esse acesso não representa a
  localização real da SC-390; conexão definitiva por Lages–Tubarão pendente.
- Traçado/superfícies próprios compartilhados entre desenho e física, sem
  incorporar assets, imagens ou coordenadas de terceiros. Escala 1:20 entre
  cidades preservada; curvas e mirante mantêm espaço para o conjunto articulado.
- Windows GNU: 66 testes do workspace e Clippy passaram. Novos testes cobrem
  tangência, inclinação, cinco inversões de direção, limites de geometria,
  apoio das doze rodas em oito pontos nos dois sentidos e contenção lateral
  pela defensa. Streaming continua limitado a nove regiões visuais residentes.
- Smokes anunciados, com dados isolados, curva e mapa local na Intel HD 4000;
  encerramento automático com código 0, seis contatos do cavalo e baú engatado.
  Capturas inspecionadas. Não houve benchmark, condução humana de ponta a ponta,
  nova execução gráfica Linux ou geração de pacote portátil. Solo externo à
  pista é decorativo e não tem colisão própria.
- Detalhes, referência municipal e instruções em
  [SERRA_RIO_DO_RASTRO.md](docs/SERRA_RIO_DO_RASTRO.md).

## Primeiro corredor interurbano e carregamento visual

- Saída norte original até um pátio de Guarapuava a cerca de 9 km do polo,
  equivalente de projeto a aproximadamente 180 km em 1:20; não é distância
  rodoviária real verificada. Inclui curvas e ondulações abaixo de 7%, terreno
  visual e colisão coerentes, e chegada plana para manobrar.
- Três destinos logísticos no treino local. F escolhe destino antes da carga,
  parado no pátio; escolha bloqueada durante o transporte. Retorno preserva os
  dois extremos. Ao reabrir, novo treino parte do pátio mais próximo e descarta
  o progresso da carga anterior. Não há economia ou fretes autoritativos ainda.
- Solo, pintura e vegetação em lotes de 400 m, limitado a nove residentes.
  Remove nós e referências de malha fora da janela; mantém materiais compartilhados,
  horizonte e pátios pequenos. Não certifica VRAM nem FPS. Colisão permanece
  residente para apoiar inclusive reboques estacionados fora da janela.
- Mapa local cobre todo o corredor e mostra os três pátios e o frete escolhido;
  Z aproxima o circuito urbano. HUD informa regiões residentes. Atlas das
  32 cidades continua sendo planejamento separado do cenário construído.
- Esse trecho cabe no limite atual de poses, sem mudança de protocolo.
  Expansão completa ainda precisa rever coordenadas, colisões e rede.
- Encerramento de rede agora aguarda brevemente a confirmação de Leave,
  retransmitindo enquanto necessário antes de desligar o transporte. A simples
  escrita no socket não garantia que a sala tivesse processado a saída.
- Validação Windows GNU: **63 testes do workspace e Clippy passaram**. Incluem
  todas as faixas de terreno, área sem buracos, orçamento por lote, remoção de nós
  ao atravessar/retornar entre regiões, escolha de destino, retomada perto do pátio,
  apoio das rodas em dez pontos dos dois relevos e sessão WebSocket com saída.
- Dois smokes gráficos anunciados antes da abertura: corredor e mapa local,
  90 quadros cada, na Intel HD 4000/OpenGL 3.3. Capturas inspecionadas, nove regiões
  carregadas e seis contatos de roda no corredor. Encerramento automático confirmado
  em `last-session.txt`; título agora identifica explicitamente janelas de teste.
  Não houve teste humano da viagem completa, benchmark, nova validação Linux ou
  regeneração do pacote portátil nesta etapa.

## Diagnóstico de encerramento

- Relato de fechamento durante direção: o proprietário esclareceu que a janela
  havia aberto automaticamente, compatível com os smokes iniciados pelo agente
  e encerrados por limite de quadros. Não foi confirmado um crash nessa sessão.
  Avisar explicitamente antes de abrir novas janelas de teste.
- Reprodução isolada com cópia da pose do proprietário: 600 quadros parado e
  1.200 com aceleração concluíram com código 0. Física estressada por 28.800 passos
  com curvas, frenagem e ré em quatro posições do terreno manteve poses finitas.
- Cliente agora registra `last-session.txt` (estado/encerramento) e, em panic Rust,
  `crash-latest.txt` com backtrace, no diretório de dados. Arquivos sobrescritos,
  limitados por caracteres e sem envio de dados. Não captura falhas nativas do
  driver; estado "executando" deixado após fechar apenas indica saída não registrada.
- Novo `--smoke-maneuvers` automatiza aceleração, curvas, frenagem e ré; depende
  de `--smoke-frames`. O script avisa sobre fechamento automático de `-Smoke`.
  Modo normal `dev.ps1` continua sem limite de quadros e informa o código de saída.
- Corrigida uma corrida independente no encerramento da rede: o worker agora
  captura a solicitação de parada antes de drenar a fila, para transmitir o Leave
  enfileirado antes de encerrar. Essa falha não explica o suposto crash ao dirigir.
- **60 testes do workspace passaram**, incluindo panic real em subprocesso com
  relatório local, estresse da física e saída da sessão WebSocket. Clippy passou.
  Build incremental do cliente concluído; sem novo pacote portátil ou novos smokes
  gráficos após o esclarecimento do proprietário.

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
| F4 — mundo | Atlas, circuito piloto, corredor norte e carregamento visual; demais corredores pendentes |
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
