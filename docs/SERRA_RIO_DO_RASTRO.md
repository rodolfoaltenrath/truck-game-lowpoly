# Serra do Rio do Rastro — primeiro trecho jogável

Versão procedural original inspirada nas curvas e no mirante da SC-390.
O [portal oficial de turismo de Bom Jardim da Serra](https://turismo.bomjardimdaserra.sc.gov.br/)
situa a serra entre Lauro Müller e Bom Jardim da Serra e descreve os cotovelos,
o relevo e o mirante. Consulta em 10/11/2026. Nenhuma imagem, modelo, textura,
base cartográfica ou sequência de curvas dessa fonte foi incorporada.

## Encontrar no jogo

Saia do polo de União da Vitória para o sul, passe pelo pátio de Porto União
e continue mais 600 m. O acesso fica no lado leste, a 1,4 km do polo, em
`z = -1400`. Entre no desvio e siga até o mirante; o retorno usa a mesma pista.
M abre o mapa local, Z mostra o corredor inteiro e R retorna ao polo.

Esse acesso é provisório para experimentar o cenário. **Não corresponde à
localização real da serra junto a Porto União.** A ligação definitiva pelo
corredor Lages–Tubarão ainda será construída; o atlas regional continua sendo
planejamento e não oferece navegação pelas 32 cidades em 3D.

## Construção

- Aproximadamente 4,8 km dirigíveis, seis travessias da encosta e cinco
  cotovelos de 180°, com raio central de 60 m. Subida e descida pelo mesmo traçado.
- Cerca de 250 m de desnível; inclinação longitudinal central limitada a 5,5%,
  com transições graduais nas extremidades. Não são altitudes reais da serra.
- Pista de 14 m incluindo acostamentos, sinalização de borda, linha central dupla,
  placas de curva e barreiras de concreto com colisão. Sugestão no HUD: 20 km/h
  nos cotovelos; é orientação de jogo, sem limitador automático.
- Mirante de chegada alargado para aproximadamente 66 m, permitindo manobras
  com o baú. Ainda sem edifício, atividade de descanso ou serviço interativo.
- Maciço facetado, taludes rochosos e horizonte serrano. Solo decorativo fora
  da pista não tem colisão própria; a superfície dirigível e as barreiras têm.
- Traçado e superfícies em `game_core::serra`, usados pelo cliente e pela física.
  Sem I/O no núcleo; criação determinística, limitada e armazenada uma vez.
- A geometria da serra acompanha os lotes visuais existentes de 400 m;
  permanece o teto de nove regiões residentes. Colisão fica residente.

A escala 1:20 continua valendo para o planejamento das distâncias entre
cidades. Este ponto turístico preserva espaço para curvas, conjunto articulado
e manobras, assim como os pátios. Não representa um levantamento da SC-390
reduzido uniformemente nem reproduz todas as suas curvas.

## Validação e próximos ajustes

Testes automatizados verificam continuidade de tangente, cinco inversões de
direção, inclinação, limite de geometria e finitude. A física verifica apoio
das seis rodas do cavalo e das seis do baú em oito pontos nos dois sentidos,
além de contenção de uma derrapagem lateral pela defensa.

Condução humana do trecho completo, ajuste da câmera perto dos paredões,
benchmark, terreno externo com colisão e validação gráfica Linux permanecem
pendentes. Uma captura curta não certifica estabilidade em viagem completa.
