# Guia de arte — low poly charmoso

O jogo **não** busca realismo. O alvo é um low poly bem acabado, coeso e
charmoso: formas simples e legíveis, cores sólidas, luz suave e quente.

## Referência

Imagens enviadas pelo proprietário em 09/11/2026 (frota de caminhões de
cabine avançada, cena com caminhão de toras em campo dourado). Servem como
**referência de estilo**: proporções, paleta, acabamento e clima. Não copiar
modelos, layouts ou detalhes específicos de nenhum pacote ou jogo.

Traços a perseguir:

- Caminhões de **cabine avançada** (sobre o motor), caixas com **quinas
  chanfradas**, sem curvas suaves.
- **Cores sólidas e saturadas** na cabine; chassi grafite; aros claros com
  cubo escuro; pneus quase pretos.
- **Para-brisa grande e claro**, grade escura com moldura clara, para-choque
  com faróis quentes embutidos.
- **Luzes acesas com brilho**: faróis brancos-quentes, lanternas vermelhas,
  piscas âmbar. Listras vermelho/branco na traseira.
- Cenário: **capim dourado**, árvores de **copa facetada** em amarelo-verde,
  pinheiros de cones empilhados, pedras claras, nuvens brancas facetadas,
  céu claro com sol brilhante.
- Luz de "estúdio ao ar livre": sol quente, céu azulado preenchendo as
  sombras, sombras macias, névoa clara no horizonte.

## Regras técnicas

Na versão nativa, as primitivas, a paleta e a montagem por material ficam em
`crates/client/src/visual.rs`. São portas da geometria original do protótipo,
com índices ajustados à orientação do OpenGL. Rodas usam 14 lados e geometria
separada; o vão inferior da cabine e os para-lamas respeitam a suspensão.

| Regra | Como |
|---|---|
| Faces planas | Toda peça em flat shading (`convertToFlatShadedMesh` ou geometria gerada com normais por face). |
| Quinas chanfradas | `Batch::cuboid` em `crates/client/src/visual.rs`; chanfro 2–6% da menor dimensão nas peças principais. |
| Poucos lados | Cilindros de 6–14 lados conforme o tamanho na tela (rodas 14, tanques 10–12, troncos 6). |
| Paleta única | Paleta original portada para `crates/client/src/visual.rs`; manter cores coesas. |
| Sem textura | Cores sólidas por material compartilhado; geometria agrupada em superfícies por cor. |
| Poucas chamadas de desenho | Peças agrupadas em `Batch`; materiais compartilhados e rodas clonadas. Solo e vegetação em até nove lotes de 400 m, com descarte por região e frustum. Instâncias de cenário seguem como próxima otimização. |
| Escala | 1 unidade = 1 metro. Caminhão ~2,5 m de largura, cabine ~3,3 m de altura total. |
| Eixos | +Y para cima, +Z para frente, +X para a esquerda (sistema destro). |
| Gerado por código ou próprio | Sem assets de terceiros sem licença registrada; nada de jogos comerciais nem mods derivados. |

## Orçamento de polígonos (alvo)

| Categoria | Triângulos |
|---|---|
| Cavalo mecânico | ≤ 6.000 (rodas incluídas) |
| Reboque | ≤ 4.000 |
| Árvore / pedra | ≤ 300 |
| Prédio pequeno | ≤ 1.500 |
| Carro de trânsito | ≤ 1.500 |

## Renderização nativa

O relevo dirigível do piloto vem de `game_core::terrain`: desenho, pintura e
colisão compartilham o perfil de altura e as curvas. Faces de solo são cortadas
nos nós do perfil antes da transformação, mantendo as faixas adjacentes sem
sobreposição. Árvores e cenário acompanham a elevação; caminhão e rodas conservam
sua geometria local e recebem a pose da simulação.

Revisão visual com abertura confirmada pelo proprietário: copas de oito
lados, pinheiros em camadas, morros decorativos, galpões e detalhes da carroceria.
A estrada usa faces horizontais adjacentes, incluindo a pintura; vegetação em
lotes de 200 m compartilha materiais. Manter a resolução nativa e medir o custo
global antes de afirmar ganhos: o cenário agora contém mais objetos.

Fyrox/OpenGL 3.3, janela em resolução nativa. Preset inicial sem SSAO, HDR,
FXAA ou sombras; G alterna sombras de 512. Não reduzir automaticamente a
resolução. Reavaliar antialiasing e brilho após medir custo e nitidez na GPU
real; o port inicial não promete equivalência de iluminação com Babylon.

## Histórico do pós-processamento web

As opções abaixo descrevem o protótipo anterior à migração, não o cliente atual.

| Qualidade | O que liga |
|---|---|
| baixa | tons ACES, FXAA, sombra 512, resolução nativa do canvas |
| média (padrão) | + brilho nas luzes (glow), sombra 1024, resolução nativa |
| alta | + bloom e MSAA 4x, sombra 2048, resolução nativa |

Tecla **G** troca; a escolha fica salva no navegador.

FXAA fica ativo em todos os presets. O brilho usa uma textura de 1/4 da
resolução; o pipeline usa buffers de cor sem HDR. SSAO foi retirado por seu
custo e pela possível granulação. A resolução interna acompanha o tamanho
CSS do canvas (escala 1), sem redução automática de pixels. Se a média do
tempo de quadro passar de 40 ms, primeiro são retirados glow, bloom e MSAA
adicional, com sombras 512; se persistir, as sombras são desligadas. O HUD
informa “efeitos reduzidos”. A tecla G restaura os efeitos do preset seguinte;
o ajuste automático não altera a preferência salva nem fica oscilando.

As rodas usam material fosco compartilhado e não recebem mapas de sombra
nas faces que giram, para evitar manchas intermitentes. Continuam projetando
sombras quando o preset as habilita. Cabine, degraus, plataforma e para-lamas
devem respeitar o volume ocupado pelos pneus, inclusive com a suspensão
comprimida. O vão dianteiro é recortado na parte inferior da cabine.

Superfícies horizontais do mapa experimental formam um mosaico no mesmo
plano: capim, asfalto e pintura substituem faces, sem planos quase
sobrepostos. Isso evita disputa de profundidade (z-fighting) nas estradas.

## Próximos passos de arte

- Reboques no mesmo estilo (baú, graneleiro, toras, tanque), com a
  mesma linguagem de chanfros e listras.
- Capim em tufos (thin instances) perto da câmera.
- Prédios/empresas de destino modulares; postes, placas e defensas.
- Variações de cabine (teto alto/baixo, defletor) por dados, não por código
  novo.
