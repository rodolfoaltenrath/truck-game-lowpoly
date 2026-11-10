# Direção e estabilidade do protótipo nativo

## Ajuste de 10/11/2026

Relato do proprietário: conjunto carregado tombava facilmente em pequenas curvas.
Reprodução automatizada: curva contínua a 60 km/h, com entrada máxima de teclado,
levou um dos corpos a ficar com sua vertical apontando para baixo. O teste
anterior de estresse verificava poses finitas, mas não detectava tombamento.

Mudanças em `truck_sim`:

- Assistência de esterço em função da velocidade: mantém até 0,6 rad parado,
  reduz o limite progressivamente e usa um modelo bicicleta aproximado com
  entre-eixos efetivo de 4,5 m e aceleração lateral alvo de 3 m/s². Esse alvo
  calcula o comando, não limita diretamente a aceleração real do conjunto.
- Suavização e limite de velocidade de esterço também ao soltar ou inverter
  a tecla. Acelerador e freio de serviço passam por transições curtas; freio
  de mão continua imediato. A lógica de freio/ré existente foi preservada.
- Molas e amortecedores recalibrados (34; compressão 3,2; retorno 4,0 nos
  parâmetros do controlador Rapier). Suspensão mais amortecida e menos balanço.
- Barras estabilizadoras por par de rodas, no cavalo e no baú: forças opostas
  proporcionais à diferença de compressão. Atuam quando ambas as rodas do eixo
  têm contato e têm força limitada. Torques externos são limpos a cada passo,
  assim como as forças externas, evitando acumular o esforço da barra.
- Massa do baú distribuída entre chassi (6.500 kg), casca (1.500 kg) e carga
  baixa (7.500 kg), com inércia calculada para o volume da carga. Preservados
  15.500 kg no reboque e 8.000 kg no cavalo; nenhuma redução de peso para obter
  estabilidade. A carga atual é fixa, não um sistema de cargas variáveis.

Os parâmetros são ajustes de jogabilidade, sem calibração a um modelo comercial
real de caminhão. Não há travamento de rotação, correção automática de orientação
ou garantia contra tombamento em colisões, encostas e manobras extremas.

## Validação

Teste de regressão cobre curvas contínuas a 30 e 60 km/h e slalom de teclado a
80 km/h, por 20 segundos simulados em cada caso, com o conjunto carregado em
piso plano. Confere inclinação dos dois corpos (vertical acima de 0,9), mudança
real de rumo, esterço sem saltos, engate, massa e manutenção de velocidade.
Essas velocidades são condições de teste, não recomendações para a serra.

Continuam os testes de frenagem, ré, engate, pernas do baú desacoplado, estresse
prolongado, apoio das rodas em relevo e contenção lateral pela defensa.
Validação humana da sensação ao volante, viagem completa na serra e gráficos
em Linux ainda são necessários. Não foi medido ganho de FPS.
