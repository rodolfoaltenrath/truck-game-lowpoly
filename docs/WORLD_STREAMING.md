# Mapa grande — primeira base da F4

> **Histórico do cliente web (commit 346aced).** O cliente atual é nativo em
> Rust. A seleção pura em `game_core::world` foi preservada, mas o streaming
> visual descrito abaixo ainda precisa ser portado. A rodovia nativa inicial
> serve para validar física e renderização; não abrir os antigos links web.

O protótipo `/?world=road&solo` usa um terreno de 32 × 32 km dividido em
regiões de 1 × 1 km. Só há conteúdo autorado num corredor reto de 12 km:
Campo Dourado, Vale Claro e um posto intermediário. Nomes, geometria e
cores são próprios do projeto; não há assets externos nesta entrega.

`game_core::world` seleciona a janela 3 × 3 ao redor do caminhão e valida
coordenadas finitas dentro de [-16000, 16000) metros. Coordenadas negativas
usam piso, mantendo as bordas consistentes. O cliente consulta a mesma
implementação em WASM, cria as regiões que entram e remove malhas e
colisores das que saem, antes do próximo passo de física.

O chão visual é um mosaico de faces sem sobreposição: asfalto e faixas
recortam a área do capim em vez de criar planos separados por centímetros.
Uma única malha por região reúne essas faces e os prédios, evitando a
disputa de profundidade que fazia superfícies distantes piscarem.

Reboques soltos preservam sua própria janela: nove regiões por área de
interesse, no máximo 36 considerando o caminhão e três reboques. As janelas
sobrepostas compartilham recursos. Assim, abandonar um reboque longe não o
faz cair quando o jogador se afasta. Jogadores remotos continuam fantasmas.
Sair dos limites devolve o caminhão e seu reboque engatado ao ponto inicial.

A pista original continua em `/`. A posição salva do mapa experimental é
separada da pista. Para ver outros jogadores no mapa experimental, todos
devem abrir `?world=road` e selecionar a mesma sala; ainda não existe
negociação de versão/mapa no protocolo.

## Como ampliar

1. **Dados e ferramenta Rust:** substituir os descritores fixos por arquivos
   com estradas em splines, relevo e prefabs. `world_tools` deve validar
   limites, gerar malhas/colisores e um manifesto com versão e hash. Manter
   uma faixa de estrada e terreno idêntica nos dois lados de cada borda.
2. **Conteúdo:** construir trechos com curvas, aclives, cruzamentos, acessos
   de empresas e paisagens low poly. Expandir por corredores jogáveis antes
   de preencher toda a área. Os povoados atuais são volumes de teste.
3. **Download por região:** buscar pacotes antecipadamente na direção da
   viagem, com cache limitado, cancelamento e fila de carga. Manter o piso
   anterior até o novo estar pronto; nunca liberar um colisor sob um veículo.
   Nesta entrega a geração é local e síncrona, sem download de pacotes.
4. **Detalhe por distância:** colisão e detalhes próximos; vegetação com
   instâncias; versões simplificadas de prédios e relevo distante. Ainda
   não há LOD ou vegetação neste mapa experimental.
5. **Escalas maiores:** medir a precisão de Rapier e render e só então
   introduzir origem móvel. Separar coordenada global da posição local,
   incluindo juntas, câmeras e remotos na troca de origem.

## Validação

Os testes Rust cobrem bordas negativas, limites, todas as 1024 células,
unicidade, continuidade da rodovia e retenção limitada para reboques.
`web/e2e/world.spec.ts` exercita direção cruzando uma borda, ida e volta
entre povoados e estabilidade das contagens de malhas e colisores.
Consultar o roadmap para saber quais testes efetivamente passaram.

F4 permanece parcial: faltam formato de arquivos, CLI, relevo, splines,
LOD, streaming assíncrono e medições de tempo de quadro/download em GPU real.
