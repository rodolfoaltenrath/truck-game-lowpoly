# Conteúdo e licenças

## Conteúdo original

O caminhão sem marca, baú, rodas, estrada, árvores, prédios e paleta são
procedurais e próprios deste projeto, sob a licença Apache-2.0 do repositório.
O port Rust registra a origem no protótipo próprio (commit 346aced).
O diagrama das 32 cidades, suas coordenadas comprimidas, a rede de ligações,
o circuito piloto e seus pátios são criações próprias baseadas no planejamento
de `docs/MAPA_SUL.md`; não incorporam traçado, imagem ou base cartográfica externa.
O corredor norte até o pátio de Guarapuava e seu relevo também são originais;
o comprimento nominal de projeto não é uma medida extraída de rodovias reais.
Nenhum modelo, textura ou som de 18 Wheels of Steel ou de mods derivados
foi incorporado. As imagens de referência não fazem parte do executável.

Nova referência de frota anexada pelo proprietário em 10/11/2026 foi usada
somente para proporção e estilo low poly. Eixos, suspensão, para-lamas,
defletor, grade e acessórios da cabine/baú são geometria própria sob Apache-2.0;
nenhum modelo, logotipo, textura ou arquivo da referência foi incorporado.

A serra procedural, suas curvas, maciço, placas, barreiras e mirante também
são originais, sob Apache-2.0. O nome e as características gerais da Serra do
Rio do Rastro foram consultados no portal municipal de Bom Jardim da Serra;
origem e limites em [SERRA_RIO_DO_RASTRO.md](SERRA_RIO_DO_RASTRO.md).

## Conteúdo incorporado pelas dependências

O arquivo `caminhao_detalhado.glb` foi fornecido pelo proprietário em
10/11/2026 como criação feita com Claude Code para este projeto. É incorporado
como conteúdo próprio sob a licença do repositório, conforme essa declaração
de origem; não foi realizada auditoria independente da geração. O original
permanece intacto. A adaptação em Rust, pneus rodoviários, céu procedural e
materiais escalares são código/conteúdo próprios. Não há texturas externas.

O [Low Poly Truck Pack de PHAZAN PRODUCT](https://assetstore.unity.com/packages/3d/vehicles/low-poly-truck-pack-179260)
está apenas em avaliação: nenhum arquivo do pacote foi incorporado. Se
comprado, registrar comprovante/licença aplicável e manter os fontes comerciais
fora do Git público (`assets-src/` e `assets/` são ignorados). O importador
atual é específico ao GLB próprio; não comprova compatibilidade com esse pack.

| Conteúdo | Origem | Licença |
|---|---|---|
| Engine e recursos internos da interface | Fyrox/Fyrox UI 1.0.1, https://github.com/FyroxEngine/Fyrox | MIT, texto em `licenses/FYROX-MIT.txt` |
| Fontes Lato Regular, Bold, Italic e Bold Italic | `fyrox-ui` 1.0.1, `src/font/*.ttf`; metadados identificam a família Lato 1.104 | SIL OFL 1.1, `licenses/LATO-OFL.txt` |

Copyright da fonte incorporada: © 2010–2011 tyPoland Łukasz Dziedzic,
Reserved Font Name “Lato”. O texto OFL distribuído pelo projeto Google Fonts
acompanha os avisos; a fonte não é vendida separadamente nem modificada aqui.

Bibliotecas Rust conservam as respectivas licenças; o empacotamento deve
incluir os avisos disponíveis nos crates usados e os textos acima. Não
confundir a licença do código do jogo com a das dependências.
