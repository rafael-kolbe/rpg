# STORY.md — registro da história

A bíblia da campanha: o que é **canon** (a história verdadeira, contada pelo
autor), o que está **em aberto** e as **ideias** ainda não decididas. As falas
vivem em `locales/<idioma>/story.ftl` (a fala *n* do diálogo `<id>` é
`dialogue-<id>-<n>`); quando acontece, onde e o que dá, em
`content/world/dialogues.json` e nas instâncias.

> **O piloto não é canon.** A história que está hoje no jogo (Corin,
> goblins de Grukk, catacumbas, selo negro, Coroa de Espinhos, Sargento Holt,
> Anciã Maren…) foi escrita só para testar o jogo e está **descartada**. Nada
> dela vale até ser recontado aqui. Ver [O piloto](#o-piloto-descartado).

## Regras

1. **Linha do tempo primeiro.** Toda data, idade ou "há X anos" entra na
   [Linha do tempo](#linha-do-tempo) e é conferida contra ela antes de virar
   fala. Por enquanto só o **ano** importa (dia, mês e estação não). Idades não
   se anotam: saem do ano de nascimento. Data que não fecha é sinalizada, nunca
   ajustada em silêncio.
2. **Fala segue a ficha.** Cada personagem fala de acordo com a própria ficha
   (personalidade, valores, voz), nunca com o humor do dia. Um personagem só
   muda por um **arco** (um acontecimento que justifique a mudança, registrado
   na ficha). Personagem ou região sem ficha ganha a ficha **antes** das falas.
   Fala que contradiz a ficha é sinalizada: ou se corrige, ou vira arco.
3. **O jogador cai num mundo que já existe.** Quase tudo o que está aqui já
   aconteceu antes do jogo começar e **não é contado de cara**. O jogador
   descobre aos poucos, pelas pessoas e pelos lugares. Cada fato tem a coluna
   *quando o jogador descobre*; nenhuma fala entrega uma revelação antes da
   hora.
4. **Lacunas são de propósito.** Nenhuma ficha precisa estar completa: um campo
   *em aberto* pode ficar assim por muito tempo, como espaço para uma revelação
   futura ligar dois pontos.
5. **Ideia não é canon.** Ideias ficam em [Ideias](#ideias) até o autor aprovar.

## Linha do tempo

O presente do jogo é **965** (20 anos depois do ataque de Volcanika).

| Ano | Acontecimento | Quando o jogador descobre |
| --- | --- | --- |
| 915 | Nasce **Brenna**, mãe de Taren. | *(em aberto)* |
| 938 | Nasce **Taren**, em Brookhollow. | — |
| 939 | Nasce **Lyra**, em Valcrest. | *(em aberto)* |
| 941 | Nasce **Tessa**, irmã de Taren. | *(em aberto)* |
| 945 | **Ataque de Volcanika.** A dragoa lendária atravessa o reino e destrói várias cidades. **Valcrest** é a que mais sofre. Em Brookhollow o fogo de mana queima os campos por duas semanas e a vila perde quase toda a colheita. **Doran, pai de Taren, morre** tentando apagar o fogo do trigal. **Corwin, pai de Jory, também morre**, de um jeito diferente. Brookhollow tinha cerca de **300** habitantes. | *(em aberto)* |
| 945 | **Os pais de Lyra morrem** no ataque de Volcanika. O Major Harlow passa a cuidar dela. | *(em aberto)* |
| 945 | A **Ordem do Lírio Rubro**, o grupo de elite mais forte do reino, chega tarde demais para impedir a destruição; persegue Volcanika e luta com ela **em outro lugar**, longe de Valcrest. Derrota a dragoa (versão oficial), mas seus membros saem **gravemente feridos**. Corre o boato de que ela fugiu e vive em outro continente. Ao saber da vitória, Taren decide se tornar o mais forte e começa a treinar. | O motivo de Taren: **mais para frente no primeiro arco**. |
| 945 | **Duas semanas depois do ataque**, quando o fogo de mana enfim se apaga, o **reino vizinho de Draskar**, hostil e intocado pela dragoa, aproveita: declara guerra e invade **Valcrest** pelos muros que Volcanika derrubou. Começa o **cerco de Valcrest**. O **Major Edmund Harlow**, de passagem pelo castelo a serviço do reino, assume a defesa e impede a queda da cidade. Halvaren manda reforços para Valcrest e outros pontos da fronteira. Surge o boato de que Draskar invocou a dragoa; nunca foi comprovado. | *(em aberto)* |
| 945–948 | **O resto da família de Lyra, incluindo o irmão mais velho, morre** na guerra com Draskar. | *(em aberto)* |
| 945–946 | A Ordem passa **1 ano** se recuperando. Sem ela, Halvaren perde cidades para Draskar e **quase perde a capital**. | *(em aberto)* |
| 946–948 | A Ordem volta e passa **2 anos** na guerra, retomando as cidades perdidas uma a uma. | *(em aberto)* |
| 948 | Pressionada pelos outros reinos que fazem fronteira com ela, Draskar declara à capital de Halvaren o **fim do confronto** e paga uma pequena indenização. Termina o cerco de Valcrest, após 3 anos de defesa de Harlow. Começa a "paz". Halvaren, desgastada pela guerra e pela destruição de Volcanika, não tem condições de retaliar. | *(em aberto)* |
| 948 ou depois *(em aberto)* | Harlow deixa Valcrest; Lyra deixa de estar aos cuidados dele. | *(em aberto)* |
| 945 em diante | O reino sobe os impostos para se reconstruir. Brookhollow para de crescer; muita gente deixa a vila fugindo da pobreza. | *(em aberto)* |
| 956 | **Lorde Cassian Velmont** visita Brookhollow procurando talentos para magia e leva Tessa, prometendo riqueza à vila se ela se destacar. Desde então, nenhuma notícia dela. | *(em aberto)* |
| até 962 | Lyra tenta se alistar no exército e é recusada por muito tempo: mulheres são raras no exército, e ela foge do estereótipo militar. Treina sozinha. | *(em aberto)* |
| 962 | Lyra desmascara um **infiltrado de Draskar** que, disfarçado em Valcrest, tentava jogar o povo contra o exército de Halvaren; o homem é preso. Por isso o exército a aceita, aos 23 anos. Desde então, ela só fica estacionada na cidade. | *(em aberto)* |
| 965 | **Início do jogo.** Taren sai de casa para se tornar o aventureiro mais forte. Deixa a mãe aos cuidados de **Jory**, amigo de infância. Brookhollow tem cerca de **100** habitantes. |
| 965 | Com a guarda ocupada com outros assuntos, um terceiro-sargento do exército dá a Lyra a missão de **reconhecimento** do acampamento goblin em Burrowood ("se você se acha tão capaz assim…"). Ela vai **sozinha** e é emboscada por goblins. Taren, a caminho de Valcrest pela estrada, ouve a luta e a salva. Juntos, cumprem o reconhecimento e destroem um acampamento goblin. | — |

## Capítulo 1 — enredo

O que acontece no jogo, em ordem. Cada cena entra aqui quando o autor a conta.

1. **Partida (Brookhollow, 965).** Taren sai de casa rumo a Valcrest.
   *(em aberto: a cena)*
2. **Burrowood.** Na estrada, Taren ouve uma luta e salva Lyra, emboscada por
   goblins durante o reconhecimento do acampamento. Juntos, cumprem o
   reconhecimento e destroem um acampamento goblin.
3. **O relatório (Valcrest).** Os dois voltam juntos para a cidade; Taren a
   acompanha até o terceiro-sargento que deu a missão. Lyra reporta o que
   aconteceu, apresenta Taren e leva uma bronca, com razão, por ter ido
   sozinha. Os dois se despedem.
4. **A Guilda (Valcrest).** Taren vai se inscrever, no salão da Guilda. A
   secretária, Delia, não dá muita atenção a ele no começo. Quando pergunta o
   objetivo dele, o salão inteiro ouve (antes disso, entre as explicações,
   ela diz que cada um só aceita missões **do próprio rank ou abaixo**: a
   regra fica plantada para a cena 8): *"o aventureiro mais forte em todo o
   mundo!"*. Todos riem, menos ela: o tom e o olhar dele mostram que não é
   vaidade. Ela não pergunta mais nada; explica como a Guilda funciona, passa o
   termo de responsabilidade para ele assinar e decide acompanhar os próximos
   passos desse novo aventureiro. Taren recebe a **insígnia de Bronze**.
5. **Os feridos.** Depois das gargalhadas, ninguém quer formar equipe com Taren.
   Antes que ele vá ao quadro de missões, uma equipe de 3 aventureiros de rank
   **Ouro** invade a Guilda: dois feridos carregam o terceiro, inconsciente, no
   meio deles. "Ajudem!" Os risos viram tensão. Delia chama os curandeiros e vai
   até eles; outro grupo de aventureiros, conhecidos dos três, pergunta o que
   poderia tê-los deixado assim numa cidade remota como Valcrest. Um dos feridos
   conta:
   - Faziam uma patrulha no **esgoto da cidade**, área de rank Bronze, aonde
     jovens aventureiros e curiosos vão treinar ou cumprir missões para
     diminuir a infestação, e às vezes precisam de ajuda.
   - Foram emboscados por um grupo de **Carniçais** (rank Ouro), que devoravam
     aventureiros novatos mortos no chão. Ninguém sabe de onde vieram; nem
     deveriam existir ali.
   - Os três estavam vencendo, até o clérigo deles levar na cabeça um golpe
     devastador de uma criatura que surgiu atrás: um **Lich** (rank Platina),
     provavelmente quem conjurou os Carniçais.
   - Os dois que restavam seguraram os Carniçais e o Lich enquanto protegiam o
     clérigo, derrotaram os Carniçais, feriram o Lich e fugiram. Chegaram à
     Guilda uns 10 minutos depois.
   - A equipe ferida são as **Lâminas Douradas**; quem pergunta são os
     **Martelos de Ferro** (ver [Gente de Valcrest](#gente-de-valcrest)).
6. **"Eu vou."** Taren diz abertamente que vai ao esgoto. Ninguém o leva a
   sério: é o novato que acabou de fazer o salão rir. Sem o jogador ver, ele
   pega do quadro uma missão de rank Bronze, caçar os ratos do esgoto, e sai
   pela porta. Só Delia percebe;
   escolhendo acreditar nele, chama alguns membros de sua confiança que estavam
   no andar de cima. *(A cena volta para Taren sem mostrar quem são.)*
7. **O esgoto, nível 1.** Corte direto do node da Guilda para o node do nível
   1 do esgoto. No canto da tela aparece uma missão: **"Mate 20 ratos"**.
   Parece tutorial; é a folha que Taren pegou do quadro. Taren entende que, se esses monstros subirem, a cidade
   corre grande risco. Se o relato das Lâminas estiver certo, os Carniçais
   morreram e o Lich está enfraquecido; talvez ele tenha até uma chance. Procura
   pistas do Lich enfrentando os monstros do nível 1. Durante essa parte da
   história, o jogador **não pode sair do esgoto**.
8. **O cabo.** Ao matar o **10º rato**, Taren reencontra Lyra, com outro soldado e um cabo, mandados ao
   esgoto pelo exército depois dos boatos sobre a Guilda.
   - O cabo manda Taren sair: o exército assume dali. Taren se recusa; pode
     ajudar.
   - O cabo: ele é só um civil e, se não obedecer, será preso. Taren mostra a
     insígnia de Bronze: não é civil, é da Guilda, e membros da Guilda podem
     aceitar missões em nome dela.
   - O cabo: só missões do próprio rank ou abaixo, e caçar um Lich é rank
     Platina. Taren: ele não falou em caçar Lich, mas os ratos do esgoto, e
     mostra o papel da missão. O cabo fica atordoado. Taren diz isso
     **literalmente**, sem malícia: é a missão que o rank dele permite, e ele de
     fato caça os ratos no caminho. O atordoamento vem de a resposta estar
     certa, não de um truque.
   - **Surpresa para o jogador:** só aqui ele entende que a missão "Mate 20
     ratos" era a folha que Taren pegou do quadro, e por quê: a regra que Delia
     explicou na cena 4.
   - Prestes a prendê-lo mesmo assim, ouve Taren dizer que não pode deixar as
     pessoas da cidade em perigo e que vai fazer o melhor para achar pistas de
     onde foi **o Lich ferido**. "O Lich está ferido?!" Taren confirma o relato
     das Lâminas.
   - Lyra aproveita a brecha: conhece Taren da missão no acampamento goblin e
     confia que ele sabe se cuidar.
   - O cabo avalia e aceita Taren na equipe de reconhecimento.
   - **Party:** Taren, Lyra, o soldado (nível 10, **convidado**) e o cabo (nível
     20, **convidado**). Primeira party de 4.
9. **O quadro (Guilda).** Corte para a Guilda: Delia olha o quadro de missões,
   percebe que a missão dos ratos sumiu e sorri.
   *(continua)*

### Mistérios em andamento

| Mistério | O que se sabe | Verdade |
| --- | --- | --- |
| **Os aliados de Delia** | Membros da Guilda de confiança dela, que estavam no andar de cima; ela os chama quando Taren vai ao esgoto. | *(em aberto: quem são)* |
| **Mortos-vivos no esgoto de Valcrest** | Carniçais (Ouro) e um Lich (Platina) surgiram num esgoto de rank Bronze, onde não deveriam existir. O Lich provavelmente conjurou os Carniçais. | *(em aberto)* |

### Mecânicas que a história pede

Regras de jogo que nascem da história; entram no `PLAN.md` quando forem
implementadas.

- **Party de 4** (a primeira é no esgoto de Valcrest).
- **Missão no canto da tela:** objetivo e contador (ex.: "Mate 20 ratos"). Uma
  missão pode surgir por história, sem o jogador escolher, e o contador pode
  disparar um evento no meio do caminho (ex.: o 10º rato).
- **Área trancada pela história:** durante um trecho, o jogador não pode sair
  de uma área (ex.: o esgoto de Valcrest).
- **Inimigos por encontro:** de 1 a 3, sorteado; quando a party passa de 3
  membros, o limite acompanha (party de 4: de 1 a 4).
- **Convidados** (*guest*): membros temporários da party que agem sozinhos; o
  jogador só escolhe as ações dos membros fixos.
- *(ideia, a decidir)* **Fugir:** o Taren não tem como vencer Carniçais ou um
  Lich nos níveis 5–8.

## Mundo

### O reino

- **Nome:** **Halvaren**.
- **Ferida:** o ataque de Volcanika em 945. Vinte anos depois, a economia e o
  coração do povo ainda carregam as sequelas.
- **Impostos:** altos desde o ataque, para a reconstrução (e para a guerra
  com Draskar).
- **Vizinho hostil:** o reino de **Draskar**, na fronteira de Valcrest. Não foi
  atingido pela dragoa; em 945 declarou guerra, tomou cidades de Halvaren, quase
  tomou a capital e cercou Valcrest por 3 anos. Em 948, pressionada pelos outros
  reinos vizinhos dela, encerrou o confronto e pagou uma pequena indenização.
  Hoje os dois reinos estão em "paz". Boato nunca comprovado: Draskar invocou
  Volcanika.
- **Nobreza:** recruta pelos vilarejos jovens com talento para magia.
- **Magia:** existe **mana**. O fogo de Volcanika era imbuído de mana pura e
  não se apagava como fogo comum.
- **Em aberto:** quem governa; capital; outros vizinhos; quem era o grupo de elite e
  o que foi feito deles (membros da Ordem do Lírio Rubro); se o boato sobre Volcanika é verdade; por que ela
  atacou.

### Guilda de Aventureiros

- **Sede:** na capital de Halvaren. Tem prédios em outras cidades, como Valcrest.
- **Entrada:** simples. O aventureiro assina um **termo de responsabilidade** que
  isenta a Guilda de qualquer responsabilidade sobre a vida dele.
- **Ranks:** num formato parecido com as patentes do exército. Cada aventureiro
  carrega uma **insígnia**, como um distintivo, que mostra o rank. Do menor ao
  maior: **Bronze, Prata, Ouro, Platina, Diamante, Oricalco** (en:
  *Orichalcum*) e **Draconita** (en: *Dracite*), o mineral mais forte desta
  história.
- **Ranks também para monstros e lugares:** cada monstro e cada área têm um
  rank, na mesma escala dos aventureiros (ex.: o esgoto de Valcrest é área de
  Bronze; Carniçal é monstro de Ouro; Lich, de Platina). Em níveis:

  | Rank | Níveis |
  | --- | --- |
  | Bronze | 1–15 |
  | Prata | 16–30 |
  | Ouro | 31–45 |
  | Platina | 46–60 |
  | Diamante | 61–80 |
  | Oricalco | 81–99 |
  | Draconita | 100+ |

  **Humanos vão no máximo até o nível 99; criaturas vão até o 130.**
- **Subir de rank:** o rank do aventureiro não segue o dos monstros que
  enfrenta. Sobe conforme ele conclui missões e alcança conquistas notáveis
  exigidas para cada rank. A insígnia de **Draconita** só é dada a quem já
  derrotou um **monstro lendário** (dragões ou outros míticos, geralmente nível
  100+).
- **Missões:** membros da Guilda são livres para aceitar missões em nome dela,
  mas só do **próprio rank ou abaixo**.
- **Draconita no reino:** só os 5 membros da Ordem do Lírio Rubro. O braço
  direito do rei está no mesmo nível, mas serve ao exército e por isso não tem
  rank da Guilda.

  Taren chega a Valcrest entre os níveis 5 e 8: um Carniçal ou um Lich pode
  matá-lo com um golpe.
- **Curandeiros:** a Guilda tem curandeiros que atendem os aventureiros.
- **Maior exemplo:** a Ordem do Lírio Rubro.

### Volcanika

- Dragoa lendária. Atacou o reino em 945.
- **Oficial:** derrotada pela Ordem do Lírio Rubro, o grupo de elite mais forte
  do reino.
- **Boato:** fugiu e vive em outro continente.

## Regiões

### Brookhollow

- **Visão geral:** vilarejo pequeno de trigo e gado que parou no tempo em
  945; gente trabalhadora, pobre e orgulhosa, que ainda lembra do fogo.
- **Geografia e clima:** um rio corta a vila e deságua no mar, que fica logo
  ao sul. Em volta, campos de trigo. Uma colina perto da vila guarda o
  cemitério. Ao norte, a floresta, no caminho para Valcrest. *(em aberto:
  clima)*
- **Vida:** plantação de trigo e pecuária. Parte da produção é vendida direto
  para **Valcrest**, mas nunca o bastante para o vilarejo crescer.
- **População:** cerca de 100 pessoas, entre idosos, adultos e crianças: um
  terço do que era em 945. Muitos morreram no ataque; outros partiram em busca
  de uma vida melhor quando a pobreza apertou.
- **Gente:** muito simples. Todos conhecem Taren desde criança e gostam muito
  dele. A vizinha da família, **Vó Hilda**, sempre divide um pouco da horta com
  eles. *(em aberto: costumes, jeito de falar, como tratam forasteiros)*
- **Poder:** **Bertram**, avô de Jory, lidera a vila. Jory será o dono do
  vilarejo quando o avô morrer. Acima deles, a Coroa e seus impostos.
- **Fé e lendas:** *(em aberto)*
- **História:** em 945 perdeu quase toda a colheita; o fogo de mana queimou os
  campos por duas semanas; Doran, pai de Taren, morreu nele. Vinte anos depois
  os campos estão replantados; não há mais marca do fogo na terra.
- **Fauna e perigos:** o maior perigo são os **goblins** da floresta ao norte,
  que sempre existiram ali (ver [Burrowood](#burrowood)).
  Fora isso, alguns animais selvagens comuns, nada fora do normal.
- **Tensões atuais:** impostos altos; estagnação há 20 anos; jovens com talento
  levados por nobres.
- **Lugares marcantes:**
  - **Praça do poço:** o centro da vila, onde todos se encontram.
  - **Casa da família de Taren:** pequena, de pedra e madeira, na beira dos
    trigais.
  - **Casa da Vó Hilda:** vizinha, com a horta que ela divide.
  - **Casa de Bertram:** a maior da vila, onde se resolvem as questões de todos.
  - **Celeiro comunitário:** onde a colheita é guardada antes de seguir para
    Valcrest, e onde o cobrador de impostos faz as contas.
  - **Moinho à beira do rio:** mói o trigo da vila.
  - **Trigais:** os campos onde o fogo de 945 queimou, replantados.
  - **Velho carvalho:** na beira dos trigais, onde Taren treina todos os dias
    desde criança.
  - **Cemitério da colina:** os túmulos de Doran, de Corwin e de quem morreu
    em 945.
  - **Foz do rio:** onde o rio encontra o mar, ao sul.
  - **Estrada norte:** sai da vila rumo à floresta e a Valcrest.
- **Ganchos:** a irmã desaparecida de Taren; o nobre que leva talentos; a
  promessa de Taren de voltar e acabar com a pobreza.

### Burrowood

- **Nome:** **Floresta de Burrowood** (*burrow* + *wood*: "mata das tocas").
- **Visão geral:** a floresta entre Brookhollow e Valcrest, atravessada pela
  estrada que liga as duas.
- **Fauna e perigos:**
  - **Goblins**, que sempre existiram ali. Procriam rápido e vivem em tocas
    subterrâneas, montes, colinas e cavernas, por isso nunca foram extintos.
  - **Animais selvagens.**
  - **Saqueadores** de Valcrest, que às vezes fogem para se esconder na
    floresta.
  - A **estrada** é um pouco mais segura que o resto da floresta.
- **Poder:** a **guarda de Valcrest** controla o número de goblins; o exército
  ajuda quando é preciso.
- **Em aberto:** todo o resto da ficha.

### Valcrest

- **Visão geral:** cidade de fronteira com o reino hostil de Draskar,
  reconstruída mas pobre, que nunca deixou de vigiar o outro lado.
  **Não é a capital.**
- **População:** cerca de 2000 habitantes: idosos, adultos, crianças,
  soldados, alguns nobres que vivem no castelo, estrangeiros de cidades
  vizinhas e aventureiros.
- **Vida:** se abastece de vários lugares; Brookhollow é só um dos fornecedores.
  Estruturalmente recuperada nesses 20 anos, mas as finanças vão mal.
- **Poder:** o **Governador Osric Penhallow** governa desde antes de 945. Os
  militares do reino o acompanham de perto: fiscalizam a restauração, vigiam
  o próprio governador e ficam atentos a um novo ataque surpresa de Draskar.
  Há um castelo onde vivem alguns nobres. A **guarda da cidade** controla o
  número de goblins de Burrowood, que nunca consegue extinguir.
- **História:** a cidade que mais sofreu em 945: a dragoa queimou casas, matou
  moradores e derrubou os muros. Draskar aproveitou, declarou guerra e invadiu
  pelos muros caídos. O cerco durou 3 anos (945–948), um verdadeiro inferno.
  A cidade só não caiu graças ao Major Edmund Harlow, que estava de passagem
  pelo castelo e comandou a defesa os 3 anos. Halvaren mandou reforços.
- **Gente:** medo e vigilância nunca deixaram a cidade, mesmo com a "paz".
- **Fauna e perigos:** a fronteira com Draskar; **crime** na cidade, por causa
  da vida difícil; saqueadores que às vezes fogem para se esconder em Burrowood.
- **Tensões atuais:** a "paz" frágil com Draskar; finanças ruins; o boato de que
  Draskar invocou a dragoa.
- **Serviços:** tem tudo de que um aventureiro iniciante precisa. A **Guilda de
  Aventureiros** tem um prédio na cidade; a sede fica na capital de Halvaren.
- **Esgoto:** tem mais de um nível. Área de rank Bronze, com uma infestação que a Guilda ajuda a
  conter; jovens aventureiros e curiosos vão lá treinar e cumprir missões.
  Fica a uns 10 minutos da Guilda.
- **Militares:** uma unidade do exército de Halvaren fica estacionada na
  cidade, além da guarda, e ajuda a guarda quando é preciso. Mulheres são
  raras no exército; as que entram se destacam pela força e pela disciplina.
- **Em aberto:** lugares marcantes.

### Gente de Valcrest

#### Figurantes da Guilda

Aventureiros que se acham mais do que são.

| Quem | Rank | Traço |
| --- | --- | --- |
| **Lâminas Douradas** (equipe) | Ouro | Nome pomposo para uma equipe que se acha a melhor da cidade. |
| Brann, líder das Lâminas | Ouro | Guerreiro; fala alto e conta vantagem até ferido. |
| Sylas, das Lâminas | Ouro | Ágil; o que conta o relato do esgoto, exagerando o próprio papel. |
| Fennick, das Lâminas | Ouro | O clérigo, inconsciente pelo golpe do Lich. |
| **Martelos de Ferro** (equipe) | *(em aberto)* | Conhecidos das Lâminas; rivais de bar que fingem não se preocupar. |
| Hogg, dos Martelos | *(em aberto)* | Grandalhão; o que pergunta o que poderia ter feito aquilo "numa cidade remota como esta". |

#### Merrick, cabo

- **Papel:** secundário; lidera a equipe de reconhecimento do exército no
  esgoto. Convidado na party (nível 20).
- **Personalidade:** apegado às regras e às ordens; teimoso, mas justo: muda de
  ideia diante de um fato novo (o Lich ferido) e de quem confia (Lyra).
- **Em aberto:** voz; o resto da ficha.

#### Tobin, soldado

- **Papel:** figurante; soldado da equipe de reconhecimento, como Lyra.
  Convidado na party (nível 10).
- **Em aberto:** traço; voz.

#### Garrick, terceiro-sargento

- **Papel:** secundário; pode aparecer mais em alguns momentos.
- **Personalidade:** disciplinado e esforçado. Por isso o esforço de Lyra
  chamou a atenção dele.
- **Relações:** conhece Lyra desde que ela entrou no exército (962). Amoleceu
  com ela ao ver o esforço dela e cedeu a missão de reconhecimento ("se você se
  acha tão capaz assim…"), sem esperar que ela fosse sozinha. Arrependeu-se
  logo, mas por dentro ficou muito impressionado. Dá a bronca quando ela volta;
  depois de um tempo, a relação dos dois volta ao normal.
- **Voz:** *(em aberto)*
- **Em aberto:** nascimento; o resto da ficha.

#### Delia, secretária da Guilda

- **Papel:** atende no prédio da Guilda em Valcrest e guia os aventureiros.
- **Personalidade:** animada e descontraída; o charme dela combina com o
  trabalho. Os veteranos da Guilda dão em cima dela (a maioria com respeito);
  ela nunca cede a ninguém, porque não tem interesse.
- **Passado:** perdeu o marido e a filha no ataque de Volcanika, em outra
  cidade do reino. Acredita, pelos boatos da época, que a dragoa sobreviveu, e
  quis virar aventureira para caçá-la. Não tinha talento para a carreira, mas
  preferiu ajudar como podia: trabalha para a Guilda, guiando aventureiros.
- **Com Taren:** foi a primeira a perceber, pelo tom e pelo olhar dele, que o
  sonho não era vaidade. Não sabe o motivo e não perguntou.
- **Voz:** *(em aberto)*
- **Revelação:** o jogador só descobre o passado dela **mais para frente**.
  *(em aberto: quando exatamente)*
- **Em aberto:** nascimento; a cidade onde perdeu a família; quando entrou para
  a Guilda.

| Quem | Canon | Em aberto |
| --- | --- | --- |
| **Edmund Harlow** | Militar do exército de Halvaren. Major em 945, de passagem pelo castelo a serviço do reino quando Draskar atacou; defendeu Valcrest durante os 3 anos do cerco e impediu a queda da cidade. Coronel em 965. | Nascimento; ficha; onde serve hoje. |
| **Osric Penhallow** | Governador de Valcrest desde antes de 945; hoje vigiado de perto pelos militares do reino. | Nascimento; ficha; por que é vigiado. |

## Personagens

### Taren (protagonista, Guerreiro)

- **Papel:** protagonista; o Guerreiro da party, na linha de frente.
- **Aparência:** *(em aberto)*
- **Nasceu:** 938, em Brookhollow.
- **Personalidade:**
  - **Forte e corajoso:** encara o perigo sem hesitar; às vezes rápido demais.
  - **Ânimo inabalável:** não desanima, nem na derrota; contagia a party, mas
    pode soar ingênuo diante de um problema sério.
  - **Gentil e sincero:** diz o que pensa e sente, sem malícia. Não mente.
  - **Inocente:** coração puro, confia fácil; pode ser enganado.
  - **Do interior:** não conhece os costumes de outros lugares nem da cidade;
    pouca instrução e pouco traquejo social.
  - **Esperto no que é prático:** entende rápido uma regra que acabou de ouvir
    e a usa na hora (ex.: a missão dos ratos). Não é malícia: ele aplica a
    regra ao pé da letra, com sinceridade.
- **Valores — o que nunca faria:** mentir; abandonar quem precisa de ajuda na
  frente dele; desistir da promessa à família; **falar por vaidade ou para se
  exibir**.
- **Medos e defeitos:** perder mais alguém por não ser forte o bastante.
  Confia em quem não devia. *(em aberto: outros)*
- **Quer × precisa:** quer ser **"o aventureiro mais forte em todo o mundo"**
  (a frase dele), para que nenhum
  dragão destrua uma cidade e mate inocentes, em lugar nenhum do mundo, e para
  tirar a família e Brookhollow da pobreza. *Precisa:* *(em aberto)*
- **Como os outros o veem:** quem não o conhece ouve "quero ser o aventureiro
  mais forte" e acha que é vaidade, exibição ou ganância. A culpa é da
  inocência dele: diz o sonho com toda a sinceridade, sem explicar o porquê e
  sem perceber como soa. **As falas dele nunca são vaidosas**: a impressão
  errada está nos olhos de quem ouve, não no que ele diz. Em Valcrest, o salão
  inteiro da Guilda ri dele; só Delia, a secretária, percebe que não é vaidade.
- **Voz:** *(rascunho, a aprovar)* direto, simples, sem palavras difíceis; fala
  com entusiasmo e trata todos de igual para igual. Fala do sonho como quem
  fala de um trabalho a fazer, nunca como quem quer aplauso. Não entende ironia
  nem etiqueta da cidade, e às vezes isso faz graça. Não fala do pai nem do
  dragão com estranhos. Por inocência, **nunca leva nada para o lado
  negativo**: provocação ou desconfiança não o ofende.
  Exemplo (canon), sobre a Ordem: *"Eles foram incríveis, mas chegaram tarde.
  E eu quero chegar antes."*
- **Relações:** a mãe, Brenna; a irmã desaparecida, Tessa; Jory, amigo de
  infância que protege a família.
  **Ordem do Lírio Rubro:** a prova de que ser o mais forte salva vidas. Taren
  quer ser **maior que a Ordem**: mesmo tendo "derrotado" a dragoa, ela não
  chegou a tempo de impedir a destruição do reino. É o **único** ponto que ele
  tem contra eles; de resto, os admira. *(em aberto: quando o jogador percebe
  essa admiração)*
- **Passado:** ver [Linha do tempo](#linha-do-tempo). Perdeu o pai no ataque de
  Volcanika; cresceu com a mãe e a irmã, passando fome. Quando soube que a
  Ordem do Lírio Rubro derrotou a dragoa, entendeu que, se fosse forte, o pai não teria
  morrido. Desde então treina todos os dias enquanto cuida da plantação e dos
  animais.
- **Revelação:** o porquê do sonho só aparece **mais para frente no primeiro
  arco**. Até lá, nenhuma fala dele ou dos outros entrega isso.
- **Arco:** *(em aberto)*
- **Segredos:** *(em aberto)*

### Lyra

- **Papel:** a primeira companheira de Taren; a **Arqueira** da party.
- **Aparência:** *(em aberto)*
- **Nasceu:** 939, em Valcrest.
- **Personalidade:**
  - **Determinada:** não desiste; transforma cada recusa em motivo para
    provar o contrário. Lado ruim: a vontade de mostrar serviço às vezes sobe à
    cabeça (como ir sozinha à primeira missão de verdade).
  - **Corajosa:** encara o perigo apesar do trauma de infância.
  - **Nojo mortal de insetos.**
- **Valores — o que nunca faria:** desistir de provar o próprio valor; recusar
  uma tarefa, por menor que seja. *(em aberto: outros)* Sabe pedir desculpas.
- **Medos e defeitos:** insetos. Ressentimento contra Draskar. Revolta por ser
  subestimada.
- **Quer × precisa:** quer ser como o Major (hoje Coronel) Harlow e provar que
  é capaz de algo maior que ficar estacionada na cidade. *Precisa:* perdoar
  Draskar (ver Arco).
- **Como os outros a veem:** o exército a subestima: mulheres são raras ali, e
  ela não se encaixa no estereótipo militar. A deixam estacionada na cidade e
  lhe dão os trabalhos mais inúteis.
- **Voz:** direta e franca, sem a rigidez de quartel:
  quer a carreira militar, mas não fala nem age como o estereótipo. Sabe pedir
  desculpas quando erra. Fica ácida quando a subestimam.
- **Na escrita:** o nojo de insetos é um bom gancho para humor, mas **com
  moderação**: nunca a mesma piada toda vez; a graça tem de vir de situações
  diferentes.
- **Relações:** **Harlow**, que cuidou dela depois de 945, até ir embora; é o
  modelo dela. Harlow **nunca a incentivou** a entrar para o exército: a
  vontade nasceu da admiração por ele. O **terceiro-sargento** que lhe deu a missão dos goblins.
  **Taren**, que a salva em Burrowood e a trata de igual para igual desde o
  primeiro minuto.
- **Passado:** sobrevivente do ataque de 945. Os pais morreram no ataque de
  Volcanika; o resto da família e o irmão mais velho, na guerra com Draskar.
  Entrou para o exército porque a família serviu, apesar do trauma. Treinou
  todos esses anos, dentro e fora do exército, e aceita qualquer trabalho na
  esperança de verem o valor dela.
- **Revelação:** *(em aberto)*
- **Arco:** um pequeno sentimento de vingança contra Draskar, que ela aprende a
  perdoar conforme a história avança.
- **Segredos:** *(em aberto)*

### Família de Taren

Os nomes podem nunca aparecer no jogo; ficam aqui para registro.

| Quem | Canon | Em aberto |
| --- | --- | --- |
| **Doran** (pai) | Fazendeiro. Morreu em 945 tentando apagar o fogo de Volcanika no trigal; não tinha como saber que fogo de mana não se apaga. | Nascimento. |
| **Brenna** (mãe) | Nasceu em 915. Criou os dois filhos sozinha. Cuida da casa enquanto Taren viaja. | Ficha. |
| **Tessa** (irmã) | Nasceu em 941. Muito talento para magia. Em 956 saiu com Lorde Cassian Velmont, que prometeu riqueza à vila se ela se destacasse; não queria deixar a família, mas confiou na promessa. Nunca mais se ouviu falar dela. | Onde está; o que aconteceu. |
| **Jory** (amigo de infância) | Neto de Bertram, o líder da vila, e futuro dono do vilarejo. Perdeu o pai, Corwin, no ataque de 945, como Taren. Taren confia muito nele. Prometeu proteger a família de Taren enquanto ele viaja. | Nascimento; ficha. |

### Gente de Brookhollow

| Quem | Canon | Em aberto |
| --- | --- | --- |
| **Bertram** | Avô de Jory; lidera a vila. | Nascimento; ficha. |
| **Corwin** | Pai de Jory. Morreu no ataque de 945, de um jeito diferente do pai de Taren. | Como morreu; nascimento. |
| **Vó Hilda** | Vizinha da família de Taren; sempre divide um pouco da horta com eles. | Ficha. |

### Outros

| Quem | Canon | Em aberto |
| --- | --- | --- |
| **Ordem do Lírio Rubro** (en: *Order of the Crimson Lily*) | O grupo de elite do reino: os **5 aventureiros mais poderosos** do reino. Cada membro tem a autoridade de um **General de Divisão** (referência: patentes do Exército Brasileiro; comanda divisões e grandes comandos). Acima deles só o **braço direito do rei**. A Ordem **não responde diretamente ao reino**; a relação é uma troca: com a autoridade conferida pelo rei, podem fazer basicamente o que precisarem e têm toda a ajuda do reino; em troca, o reino exige **prioridade** nos assuntos do reino, e as **missões especiais** são entregues somente a eles. A Ordem não recebe pagamento do reino, mas **fica com toda a recompensa das missões**, exceto itens especiais que o rei pediu para recuperar. Algo como **corsários** a serviço da coroa, com a diferença de que o motivo deles não é o dinheiro. Termo para a autorização: proposta *Carta de Corso* (en: *Letter of Marque*), a definir. **Reputação:** para o povo, um grupo heroico, o maior exemplo de heroísmo da Guilda de Aventureiros do reino. Ninguém os vê como vilões. Nome curto: **Lírio Rubro** (en: *Crimson Lily*). O exército do reino tem outros no nível de General de Divisão (detalhar depois). Em 945 chegou tarde demais para impedir a destruição; perseguiu Volcanika e lutou com ela em outro lugar (onde: em aberto). Derrotou a dragoa (versão oficial), saindo gravemente ferida; levou 1 ano para se recuperar e, de volta em 946, retomou uma a uma as cidades que Draskar tinha tomado. **Emblema:** um lírio vermelho-sangue atravessado de cima a baixo por uma espada fincada, cuja lâmina serve de sustentação ao caule. | Quem são os 5 (e se são os mesmos desde 945); o que foi feito deles; quem é o braço direito do rei; se o reino usa esses nomes de patente ou só a equivalência. Proposta: lema *"Pelo aço, a flor se ergue."* (en: *"By steel, the flower rises."*) |
| **Lorde Cassian Velmont** | Nobre que levou Tessa de Brookhollow em 956, procurando talentos para magia. | Tudo o mais. |

## Ideias

Propostas, **não canon**. Marque o que gostar, mude ou descarte.

- **B1 — O nobre e a irmã:** a busca da nobreza por talentos para magia não é
  só para formar magos; ela serve a algum plano. A irmã pode estar presa, sendo
  usada, ou ter escolhido o outro lado.
- **B2 — Para onde vão os impostos:** o reino cobra para se reconstruir, mas
  Valcrest, a mais atingida, continua marcada. Parte do dinheiro vai para outro
  lugar.
- **B3 — Volcanika viva:** o grupo de elite não matou a dragoa; fez um acordo,
  ou a selou. A versão oficial é uma mentira que sustenta o poder de alguém.

## Modelos de ficha

### Personagem

- **Papel:** na história e na party.
- **Aparência:** o que se vê em 3 segundos (silhueta, cor, um detalhe marcante).
- **Nasceu:** ano (entra na linha do tempo).
- **Personalidade:** 3 a 5 traços, com o lado bom e o lado ruim de cada um.
- **Valores — o que nunca faria:** os limites que a fala não pode cruzar.
- **Medos e defeitos.**
- **Quer × precisa:** o objetivo que persegue e o que de fato precisa aprender.
- **Como os outros o veem:** a primeira impressão, certa ou errada.
- **Voz:** vocabulário, ritmo, tiques, como trata cada pessoa, o que nunca diz.
  Duas ou três falas de exemplo.
- **Relações:** com cada personagem importante.
- **Passado:** o que importa para a história.
- **Revelação:** o que o jogador descobre dele, e quando.
- **Arco:** como pode mudar, e o que teria que acontecer para isso.
- **Segredos:** o que só nós sabemos.

### Região

- **Visão geral:** em uma frase, a sensação do lugar.
- **Geografia e clima:** relevo, estações, como é andar por lá.
- **Vida:** do que vivem, rotina, comida, festas, o que se compra e vende.
- **Gente:** como são, costumes, jeito de falar, como tratam forasteiros.
- **Poder:** quem manda, formal e de fato.
- **Fé e lendas:** o que acreditam, histórias que contam.
- **História:** o que aconteceu ali (entra na linha do tempo).
- **Fauna e perigos:** o que vive ali e por quê; é a fonte dos monstros do
  lugar (nenhum monstro sem motivo para estar ali).
- **Tensões atuais:** o que está errado agora.
- **Lugares marcantes:** os nodes do mapa e o que significam.
- **Ganchos:** personagens e histórias que nascem daqui.

## Nomes

**Canon:** Taren, Lyra, Brookhollow, Valcrest, Burrowood, Volcanika, Ordem do Lírio Rubro; ranks da Guilda: Bronze, Prata, Ouro, Platina, Diamante, Oricalco, Draconita.

**Propostos (provisórios, a aprovar):** Halvaren (o reino); Doran (pai),
Brenna (mãe), Tessa (irmã), Jory (amigo de infância); Lorde Cassian Velmont (o nobre recrutador);
Bertram (avô de Jory, líder da vila), Corwin (pai de Jory), Vó Hilda (vizinha);
Draskar (o reino hostil); Edmund Harlow (o militar que defendeu Valcrest),
Osric Penhallow (governador de Valcrest); Garrick (o terceiro-sargento), Delia
(secretária da Guilda); Merrick (o cabo), Tobin (o soldado); figurantes:
Brann, Sylas, Fennick (Lâminas Douradas), Hogg (Martelos de Ferro).

## O piloto (descartado)

O jogo ainda roda a história do piloto, que **não é canon** e será substituída
conforme a história verdadeira for escrita. Ela é útil só como inventário do
que trocar:

- **Contradiz o canon:** Valcrest como capital; Taren sem família e sem
  passado; a abertura com a Anciã Maren; falas de Taren que soam exibidas.
- **Personagens do piloto, fora do canon até serem recontados:** Corin,
  Anciã Maren, Fazendeiro Carvel, Sargento Bram Holt, atendente da Guilda,
  Grukk, Guardião dos Ossos, Coroa de Espinhos.
- **Lugares do piloto:** Floresta de Thornwood e as Catacumbas sob Valcrest;
  os nodes de Brookhollow e Valcrest também foram inventados para o piloto.
- **Mecânica que fica:** mapas, batalhas, party, lojas e saves não dependem da
  história; só o conteúdo (`content/world/`, `story.ftl`) muda.
