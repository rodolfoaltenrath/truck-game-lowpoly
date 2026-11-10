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

| Regra | Como |
|---|---|
| Faces planas | Toda peça em flat shading (`convertToFlatShadedMesh` ou geometria gerada com normais por face). |
| Quinas chanfradas | `chamferBox` em `web/src/render/geometry.ts`; chanfro 2–6% da menor dimensão. |
| Poucos lados | Cilindros de 6–14 lados conforme o tamanho na tela (rodas 14, tanques 10–12, troncos 6). |
| Paleta única | Toda cor sai de `web/src/render/palette.ts`. Cor nova entra lá, com nome. |
| Sem textura | Cor por vértice. Material próprio só para o que muda em jogo (pintura, luzes) ou tem brilho especial (vidro). |
| Poucas chamadas de desenho | Peças de um objeto fundidas com `mergeParts`; objetos repetidos (árvores, pedras) com thin instances. |
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

## Pós-processamento (`web/src/render/graphics.ts`)

| Qualidade | O que liga |
|---|---|
| baixa | tons ACES, sombra 1024 |
| média (padrão) | + FXAA, brilho nas luzes (glow), sombra 2048 |
| alta | + bloom, MSAA 4x, oclusão de ambiente (SSAO), sombra filtrada alta |

Tecla **G** troca; a escolha fica salva no navegador.

## Próximos passos de arte

- Reboques no mesmo estilo (baú, graneleiro, toras, tanque), com a
  mesma linguagem de chanfros e listras.
- Capim em tufos (thin instances) perto da câmera.
- Prédios/empresas de destino modulares; postes, placas e defensas.
- Variações de cabine (teto alto/baixo, defletor) por dados, não por código
  novo.
