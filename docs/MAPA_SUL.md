# Mapa do Sul — proposta inicial

Estado: projeto de mundo, ainda não implementado no cenário 3D. Referências
enviadas pelo proprietário em 10/11/2026; revisão de serviços e ligações abaixo
é uma proposta de jogabilidade, sujeita a ajuste no próximo desenho.

## Direção adotada

- Paraná, Santa Catarina e Rio Grande do Sul, com distribuição geográfica
  relativamente fiel e distâncias comprimidas para jogar.
- As 30 cidades do desenho são mantidas; União da Vitória (PR) e Porto União
  (SC) entram como cidades adicionais. Total: **32 cidades: PR 11, SC 11, RS 10**.
- União da Vitória/Porto União formam o polo de início compartilhado do
  multiplayer, com dois centros urbanos e uma área logística inicial comum.
- Traçados e cenários próprios. O desenho é referência de organização;
  não estabelece posições exatas, escala métrica ou numeração definitiva.
- Laranja: corredores principais; amarelo: ligações secundárias, variantes
  e atalhos. Ambas as categorias devem oferecer escolhas de percurso.
- Todas as cidades listadas podem ter origem/destino de cargas. Tamanho ou
  cor do círculo não limita acesso a fretes; distinguir capitais, polos e
  cidades menores pela importância visual.
- Ícones separados para combustível, oficina, descanso e pedágio. Um mesmo
  complexo pode reunir serviços, sem repetir ícones sobrepostos no mapa.

## Cidades

| UF | Cidades propostas |
|---|---|
| PR (11) | Curitiba, Londrina, Maringá, Cascavel, Foz do Iguaçu, Ponta Grossa, Guarapuava, Apucarana, Umuarama, Paranaguá, **União da Vitória** |
| SC (11) | Florianópolis, Joinville, Blumenau, Itajaí, Chapecó, Lages, Criciúma, Tubarão, São Miguel do Oeste, Jaraguá do Sul, **Porto União** |
| RS (10) | Porto Alegre, Caxias do Sul, Pelotas, Rio Grande, Santa Maria, Passo Fundo, Cruz Alta, Uruguaiana, Bagé, Ijuí |

## Rede a desenhar

Cada linha abaixo representa uma sequência de ligações do jogo, com curvas,
entroncamentos e acessos intermediários a desenhar. Não significa que todos
os centros urbanos estejam na mesma BR ou ligados por uma estrada direta.
Localidades intermediárias podem aparecer como cenário sem virar cidades
de frete. Os corredores se cruzam nas cidades repetidas, formando uma rede
única e circuitos alternativos.

| Corredor de jogabilidade | Sequência de polos |
|---|---|
| Travessia do Paraná | Foz do Iguaçu — Cascavel — Guarapuava — Curitiba — Paranaguá |
| Norte do Paraná | Cascavel — Umuarama — Maringá — Apucarana — Londrina |
| Ligação norte/centro | Apucarana — Ponta Grossa — Curitiba; Ponta Grossa — Guarapuava |
| Saídas do polo inicial | Guarapuava — União da Vitória — Porto União — Lages; União da Vitória — Curitiba |
| Interior catarinense | Porto União — Chapecó — São Miguel do Oeste; Chapecó — Lages |
| Norte catarinense | Porto União — Jaraguá do Sul — Joinville; Jaraguá do Sul — Blumenau — Itajaí |
| Litoral norte/sul | Curitiba — Joinville — Itajaí — Florianópolis — Tubarão — Criciúma — Porto Alegre |
| Litoral/planalto | Florianópolis — Lages; Lages — Caxias do Sul — Porto Alegre |
| Oeste e norte gaúcho | São Miguel do Oeste — Ijuí — Cruz Alta — Passo Fundo; Chapecó — Passo Fundo |
| Interior gaúcho | Ijuí — Santa Maria — Uruguaiana; Cruz Alta — Santa Maria; Passo Fundo — Caxias do Sul |
| Alternativas gaúchas | Passo Fundo — Porto Alegre; Santa Maria — Porto Alegre; Uruguaiana — Porto Alegre |
| Sul gaúcho | Uruguaiana — Bagé — Pelotas — Rio Grande; Santa Maria — Bagé; Porto Alegre — Pelotas |

A primeira revisão deve conferir separadamente geografia, conectividade e
legibilidade: entroncamento real usa nó; linhas que só passam por cima usam
ponte/viaduto. Evitar atalhos em que o caminhão cruza rios ou serras sem uma
travessia construída. O polo inicial não pode ser a única conexão PR–SC.

### Numeração das estradas

BR-101, BR-116, BR-153, BR-277, BR-282 e BR-290 são referências para os
corredores principais. A ligação Curitiba–Joinville precisa considerar
BR-376/BR-101, em vez de atribuir a BR-476 a todo esse caminho. Uma ligação
com nomes de cidades nos dois extremos pode exigir mais de uma rodovia e
acessos urbanos; conferir trecho por trecho antes de colocar placas.

A BR-277 é referência para o eixo Paranaguá–Curitiba–oeste do Paraná–Foz.
A BR-282 é referência para Florianópolis–Lages–oeste catarinense, com acessos
para os destinos selecionados. Não transportar automaticamente os escudos
de rodovia, a escala gráfica ou as posições de serviços do rascunho.

## Polo inicial multiplayer

- Uma garagem/transportadora original em área de acesso fácil entre os dois
  polos urbanos. União da Vitória e Porto União continuam com identidades e
  destinos distintos, conectados por ruas e acessos próprios.
- Prever **16 vagas de saída** no pátio para a capacidade atual da sala,
  distribuídas em baias distintas. A posição de cada vaga deve acomodar
  cavalo e reboque sem sobreposição, com corredor de manobra e saída livre.
- Todos os jogadores novos começam nesse polo, com combustível, oficina,
  estacionamento, área de descanso e acesso ao quadro de fretes próximos.
- O servidor deverá atribuir/reservar vagas e liberar a reserva após a saída;
  isso é trabalho futuro de sessão/jogabilidade. No MVP, jogadores continuam
  sem colisão entre si, conforme a decisão já adotada no projeto.
- Reconexão deve preservar a posição válida da sessão; não mandar todos de
  volta à garagem a cada oscilação de rede. Persistência entre sessões fica
  alinhada à carreira da F7.
- Três primeiras opções de frete: percurso urbano curto entre os dois polos,
  saída para Guarapuava e saída para o planalto catarinense. As duas últimas
  devem oferecer curvas fáceis e trechos de prática antes de introduzir
  serras mais difíceis. Empresas, cargas e pagamentos dependem de F5/F6.

## Serviços — posicionamento proposto

Começar com **12 complexos de apoio**, sem assumir que correspondem a postos
ou empresas reais. A tabela define áreas aproximadas; posições exatas só
serão fechadas após comprimir e medir a rede dirigível.

| Área | Serviços propostos | Função |
|---|---|---|
| União da Vitória / Porto União | combustível, oficina, descanso, garagem e fretes | Polo inicial, acessível antes de qualquer viagem longa |
| Cascavel — acesso rodoviário | combustível, oficina e descanso | Cobrir o oeste paranaense e a viagem para Foz |
| Apucarana — entroncamento do norte | combustível e descanso | Apoiar Maringá/Londrina e a descida ao centro do PR |
| Curitiba — contorno logístico | combustível, oficina e descanso | Apoiar litoral, planalto e corredor para SC |
| Chapecó — acesso urbano | combustível, oficina e descanso | Apoiar o oeste de SC e o caminho ao RS |
| Lages — entroncamento do planalto | combustível, oficina e descanso | Apoiar interior/litoral e travessia norte/sul |
| Joinville / acesso a Jaraguá do Sul | combustível e descanso | Apoiar litoral norte e a ligação ao polo inicial |
| Tubarão — acesso rodoviário | combustível e descanso | Apoiar o litoral sul catarinense |
| Passo Fundo — entroncamento regional | combustível, oficina e descanso | Apoiar conexões com SC, Ijuí e Caxias |
| Santa Maria — acesso rodoviário | combustível, oficina e descanso | Apoiar o interior gaúcho e as opções para oeste/sul |
| Porto Alegre — contorno logístico | combustível, oficina e descanso | Apoiar os eixos de Caxias, litoral e Pelotas |
| Pelotas — entroncamento para Rio Grande/Bagé | combustível, oficina e descanso | Apoiar os destinos do extremo sul |

Regras de revisão:

1. Medir distância/tempo até combustível em **todos** os caminhos, incluindo
   atalhos. Corrigir principalmente as pontas de Foz, Umuarama, Londrina,
   Paranaguá, São Miguel do Oeste, Uruguaiana, Bagé e Rio Grande. Se necessário,
   acrescentar bomba/área de apoio menor, sem criar outra cidade.
2. Dimensionar intervalos conforme consumo e capacidade do caminhão inicial.
   Nenhuma partida deve depender de alcançar um posto com o tanque quase
   vazio. Ainda não há consumo implementado para validar esses intervalos.
3. Postos e oficinas ficam em recuos com entrada e saída; bombas, cancelas e
   caminhões estacionados não ocupam a faixa de circulação. Prever retorno
   ou acesso pelos dois sentidos onde a rede exigir.
4. Colocar descanso antes/depois de viagens longas e serras, aproveitando
   complexos existentes. Não definir penalidade de fadiga nesta proposta.
5. Pedágios ficam para a revisão dos corredores principais. Não colocar
   cobrança obrigatória na única saída da garagem ou no primeiro percurso
   urbano. Se uma cobrança representar trecho real, conferir sua localização;
   valores econômicos do jogo serão definidos nas regras autoritativas.

## Escala e construção

Não fixar a escala da imagem anexada. Usar inicialmente metas de **tempo de
viagem**, propostas para teste: frete inicial 3–5 minutos, regional 8–15 e
longa distância 20–40. Ajustar no mundo dirigível com o caminhão atual.
Veículos e largura de pista continuam em metros reais; comprimir os espaços
entre cidades, preservando curvas legíveis e áreas de desaceleração.

Primeiro bloco de cenário: pátio comum, acessos dos dois municípios, um
circuito curto e um posto/oficina. Depois, uma saída interurbana e uma rota
alternativa que voltem ao polo. Só então ampliar os corredores por regiões.
Não carregar as 32 cidades de uma vez. Fazer orçamento de malhas, colisores
e efeitos por região e medir na máquina do proprietário antes de expandir.

Ainda pendentes: coordenadas, escala, splines, relevo, posicionamento exato
de serviços, prefabs, licenças de eventuais dados geográficos incorporados,
empresas/cargas e implementação da política de início no servidor/cliente.

## Referências e origem

- Dois desenhos enviados pelo proprietário nesta conversa (10/11/2026):
  referências de conceito; os arquivos de imagem não foram incorporados ao
  jogo ou ao Git. A proposta de rede e de serviços deste documento é própria.
- [DNIT — Mapas Multimodais](https://www.gov.br/dnit/pt-br/assuntos/planejamento-e-pesquisa/dnit-geo/mapas-multimodais/mapa-do-brasil): conferência dos corredores federais.
- [Ministério dos Transportes — Sistema Federal de Viação](https://dados.transportes.gov.br/dataset/2de48067-ccb4-42ac-8646-7f12232f8726/resource/c7351314-5097-4118-88d3-9b77e7fee547/download/sfv_rodoviario.pdf): referência de itinerários BR-277/BR-282; não representa validação atualizada de cada ligação proposta.
- [DER-PR — Consulta da malha rodoviária](https://www.der.pr.gov.br/servicos/Servicos/Rodovias-e-Travessias/Consultar-a-malha-rodoviaria-do-Parana-ybrzlAN4), [SIE-SC — Mapoteca](https://geo.sie.sc.gov.br/docs/index.php?artigo=mapoteca) e [DAER-RS — Mapas](https://daer.rs.gov.br/mapas): fontes para conferir as próximas versões do traçado.
