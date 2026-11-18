# RPG de turno (ATB) — Planejamento

> 2026-11-17 · rascunho inicial

Um RPG de navegador com batalha por turnos em tempo ativo (ATB), no espírito do
Final Fantasy 7, publicado em `rpg.shiraori-games.com`: Modo História
single-player e Modo Online co-op, com todas as regras rodando no servidor.
O jogo é em inglês (padrão), com tradução para português nas configurações.

**Glossário** — termos do plano e o nome usado no código e no jogo (inglês).

| Termo | Código / jogo | Significado |
| --- | --- | --- |
| Mapa-múndi | `WorldMap` | O mundo inteiro, dividido em peças |
| Peça | `Region` | Uma região do mapa-múndi, de borda arredondada; cada peça tem uma instância |
| Instância | `Instance` | A área de uma peça vista de perto: nodes ligados por trilhas |
| Node | `Node` | Um ponto clicável dentro de uma instância (batalha, baú, NPC, saída, cidade…) |
| Trilha | `Trail` | Ligação tracejada entre dois nodes; pode estar bloqueada (item, quest, evento) |
| Safe zone | `SafeZone` | Zona de instância com 0% de chance de batalha |
| Emboscada | `Ambush` | Ataque de surpresa: ATB cheia para quem embosca; o lado emboscado começa **Distraído**. Funciona dos dois lados |
| Distraído | `Distracted` | Status: o próximo golpe recebido causa 150% de dano. Termina ao levar esse golpe ou quando a própria ATB chega a 100%. Visual: avatar virado de costas |
| Modo História | `StoryMode` | Campanha single-player com os heróis fixos da história |
| Modo Online | `OnlineMode` | Multiplayer, com Guerreiros recrutados na Guilda |
| Herói | `Hero` | Personagem fixo da história: classe, habilidades e crescimento definidos pelo jogo |
| Guerreiro recrutado | `Recruit` | Personagem criado pelo jogador na Guilda de Aventureiros, só para o Modo Online |
| Mastery Tree | `MasteryTree` | Grade de habilidades: libera casas vizinhas com pontos de Mastery; a casa dá a permissão, o item/magia ainda precisa ser obtido |
| Templo | `Temple` | Prédio de cidade onde o jogador salva o ponto de ressurreição (de graça) |
| Untradeable / Tradeable / Temporário | `Untradeable` / `Tradeable` / `Temporary` | Origem do item: história, Online, ou item com prazo |

---

## 1. Visão geral

Cada personagem tem uma barra que enche com o tempo e, quando cheia, libera um
comando. O tempo não espera ninguém (no modo Ativo), então decidir rápido
importa — mas sem exigir reflexo de jogo de ação.

**Pilares**

- **Batalha é o jogo.** Tempo corre, decisões rápidas importam.
- **Simples de ler.** Poucos atributos, poucas fórmulas, números visíveis na tela.
- **O servidor é a verdade.** Toda a simulação (batalha, mapa, itens, save) roda
  no servidor, nos dois modos — inclusive no Modo História. Sem isso, trapacear
  fica liberado para quem entende. Consequência: o jogo exige estar conectado.

**Modos de jogo**

| | Modo História | Modo Online (multiplayer) |
| --- | --- | --- |
| Personagens | Heróis fixos da história | Guerreiros recrutados na Guilda de Aventureiros |
| Party | Quase sempre 3; um NPC da história pode entrar temporariamente e levar a 4 | Máximo 3, sem exceção |
| Classe e habilidades | Fixas por herói, árvore de habilidades fixa | Escolhidas pelo jogador no recrutamento, fixas depois disso |
| Evolução de atributos | Automática, ritmo fixo por classe | O jogador distribui os pontos (mesma escala de poder dos heróis) |
| Jogadores | Só single-player | Co-op |
| Liberação | Desde o início | Após certo ponto da história (por volta do nível 30; detalhar depois) |
| Exploração | Mundo inteiro, conforme a história avança | Só as regiões já liberadas no Modo História |
| Guerreiros por conta | — | Até 15 |
| Itens e dinheiro | Só pode usar itens **Untradeable** | Usa Untradeable e Tradeable; tudo que se ganha no Online nasce Tradeable (inclusive itens de evento) |
| Chat | Desligado | Global, party, mensagem privada e até 3 canais customizados por jogador |
| Mistura | Guerreiros recrutados **não** entram no Modo História | — |

**Untradeable:** tudo que vem da história recebe essa tag — dinheiro, itens,
poções, equipamentos. Item Untradeable não pode ser negociado com outros
jogadores. O Modo História só usa itens Untradeable; o Modo Online usa os dois
tipos.

**Temporário:** exceção rara (manter no mínimo possível). Item com prazo de
validade: não é Tradeable e não pode ser usado no Modo História.

**Escopo**

| | MVP (single-player) | Depois (web, multiplayer) |
| --- | --- | --- |
| Batalha | ATB completo, 3 heróis contra até 4 inimigos | Cada jogador controla 1+ heróis da mesma party |
| Conteúdo | 3 classes, ~8 inimigos, 1 chefe | Mais classes, áreas, equipamentos |
| Progressão | XP e nível, conta e save no servidor | Guerreiros recrutados, trocas entre jogadores |
| Mundo | Mapa-múndi com trilhas, cidades e calabouços clicáveis | Lobby e salas de party |

Fora do escopo por enquanto: movimentação livre do personagem, PvP,
monetização.

---

## 2. Telas e estilo visual

O jogo é **ilustração + interface + efeitos**: nada de sprites animados nem de
personagem andando pela tela. Toda navegação é por clique em pontos de
ilustrações vistas de cima.

### Níveis de navegação

```
Mapa-múndi  ──clica na cidade──▶  Mapa da cidade  ──clica na Loja de Armas──▶  Janela da loja
  (trilhas)                        (prédios clicáveis)
     │                                    │
     └─clica no calabouço─▶ Mapa do calabouço (nós + trilhas) ──fala com NPC──▶ Cena de diálogo
```

| Tela | O que é | Interação |
| --- | --- | --- |
| Mapa-múndi | Ilustração do mundo inteiro (escala de um mapa de SNES), dividido em **peças** de borda arredondada; regiões: floresta, deserto, oceano, praia, montanha, neve, cavernas | Andando a pé, passar por uma peça sempre abre a **instância** dela |
| Mapa da cidade | Visão detalhada da cidade, ainda de cima | Locais clicáveis: Loja de Armas, Loja de Poções, Guilda de Aventureiros, Hotel, Casas, entrada de Calabouço… cada um abre uma janela de interface ou uma cena de diálogo |
| Instância (campo, floresta, calabouço, zona de exploração…) | A área daquela peça do mapa-múndi vista de perto | Pontos ligados por trilhas **tracejadas**, algumas bloqueadas até ter um item, quest ou evento; pontos de batalha, baú, NPC, saída, chefe |
| Cena de diálogo | Fundo do local (interior da casa, caverna, floresta) + avatar do NPC num canto + caixa de diálogo embaixo, estilo visual novel | O avatar do jogador **nunca** aparece; escolhas de resposta na própria caixa |
| Batalha | Party de um lado, inimigos do outro, avatares estáticos; menu embaixo no estilo FF7 (design próprio) | Comandos no menu; barras ATB ao lado de cada herói |
| Janelas de menu | Abrem por cima do mapa: Status/Atributos, Habilidades, Inventário, Equipamento | Interface pura; disponíveis fora de batalha |

### Mundo, instâncias e fog of war

**Peças.** O mapa-múndi é dividido em regiões de formato orgânico, com bordas
arredondadas — não peças de quebra-cabeça com encaixes. Cada peça é uma
**instância**. O mundo vai se "montando" conforme o jogador explora.

**Instâncias fixas e coerentes.** Instâncias não são geradas aleatoriamente: são
sempre as mesmas. E o desenho de cada uma **tem que bater** com a área da peça no
mapa-múndi: estrada de leste a oeste no mundo = estrada de leste a oeste na
instância; nenhum lago que não exista no mundo.

Como isso é garantido por construção:

- **Uma única fonte de geografia** para o mundo inteiro: oceano, rios, lagos,
  estradas, florestas, montanhas, desertos, guardados como formas vetoriais.
- **Mapa-múndi** = essa geografia vista de longe, pouco detalhe.
- **Instância** = a mesma geografia recortada na área da peça e ampliada, com
  detalhe por cima (árvores individuais, pedras, cercas, construções). O detalhe
  vem de uma **semente fixa por peça**: sempre igual, sem desenhar árvore por
  árvore à mão.
- **Feito à mão por instância:** os pontos, as trilhas, os bloqueios e o conteúdo
  de cada ponto.

**Fog of war** — a mesma regra no mapa-múndi (por peça) e nas instâncias (por
ponto):

| Estado | Mapa-múndi (peça) | Instância (ponto) | Como chega nele |
| --- | --- | --- | --- |
| Oculto | Névoa densa | Névoa densa | Padrão |
| Meia névoa | Terreno visível e esmaecido, sem conteúdo | Ponto visível e esmaecido, sem saber o que é | Vizinho direto de onde o jogador está |
| Explorado | Visível por completo | Visível por completo, com tipo e conteúdo | Já passou por ali alguma vez |
| Terreno revelado | Terreno visível, sem conteúdo | Terreno visível; nodes e conteúdo continuam desconhecidos | Mapa (loja, drop, baú — um mapa pode revelar várias peças de uma vez) ou passar por ali de carruagem, montaria ou barco |

**Só andar a pé explora os detalhes** (nodes e o que há neles). Mapas e
transportes revelam apenas o terreno.

Névoa em SVG: máscara com borda suave e textura de nuvem.

### Viagem

| Modo | Rota | Abre instâncias no caminho? | Ataques no trajeto | Chegada | Particularidades |
| --- | --- | --- | --- | --- | --- |
| A pé | Livre, peça por peça | **Sim, sempre** — obrigatório atravessar cada instância | Pelas regras de encontro da instância | Ponto de entrada da borda por onde veio | — |
| Carruagem | Roteiro fixo (como um ônibus) | Não: movimento automático mostrado no mapa-múndi, sem clicar em nada até chegar | Sim; após a batalha a carruagem segue viagem | Node fixo do destino | Morrer no trajeto = carruagem some. Revela o terreno por onde passa |
| Montaria | Escolhida pelo jogador no mapa-múndi | Não | Sim | Instância abre no node de entrada conforme a direção de onde veio | Alugada (vai embora ao chegar) ou própria (fica com o jogador). Revela o terreno por onde passa. Terrenos proibidos, já perceptíveis na meia névoa |
| Barco | Porto a porto, itinerário fixo por cidade | Não (viagem pelo mar) | Depende da rota: algumas seguras, outras não | Porto de destino | Cada porto liga só aos próximos (1 a 3 saltos); às vezes precisa baldeação (inspirado no sistema de barcos do Tibia). Revela o terreno por onde passa |
| Teleporte | Direto de um ponto a outro do mapa-múndi | Não | Não | Abre a instância do destino, sempre no mesmo node | Fontes: "Pergaminho para <destino>" (raro, destino fixo), Templos e Escolas de Magia das cidades, NPCs especiais (destinos específicos) |

**Montaria em peça oculta:** pode entrar, mas o jogador é desmontado
automaticamente e aparece no primeiro node de entrada daquela instância
(conforme o lado por onde chegou).

**Comida da montaria própria:**

- Montado: 1 comida a cada 3 peças do mapa-múndi.
- Desmontado (montaria acompanhando a pé): 1 comida a cada 30 nodes. Emboscadas
  contam como node — mesmo parado, o bicho come.
- Sem comida, ela vai embora.

**Morte e ressurreição:** o jogador ressuscita no último **Templo** salvo (como
no Tibia). Carruagem e montaria alugada desaparecem; montaria própria volta para
o estábulo da cidade desse Templo. Salvar no Templo não custa nada. Penalidade
por morte: a decidir.

**Ataques durante a viagem (carruagem, montaria, barco):** cada rota tem a sua
própria tabela de encontros, formada pelos monstros das peças por onde passa. Em
casos especiais e raros, um monstro que **não** pertence àquelas peças pode
emboscar a viagem (ex.: um dragão surgindo do nada no meio do caminho).

**Encontros:**

- Cada trilha tem uma **chance fixa de batalha, de 25% a 100%**, definida no
  conteúdo. A chance **não sobe** conforme se anda (diferente do Pokémon).
- **Safe zones:** zonas de instância com 0% por padrão.
- **Repelente:** zera a chance, mas **só contra monstros de nível inferior à
  média da party**.
- **Ordem dos sorteios ao andar de um node para outro:**
  1. chance de encontro da trilha (25–100%);
  2. se houve encontro, chance de **emboscada inimiga** (5% por padrão).

  Andando, só os inimigos emboscam; o jogador nunca embosca ao caminhar.
- **Emboscada inimiga:** ATB 100% para os inimigos, heróis com ATB 0% e
  Distraídos.
- **Emboscada do jogador:** só clicando em "Emboscada" num node. A batalha começa
  sempre; o **efeito de emboscada acontece com 20% de chance** (base; aumenta com
  itens, habilidades e outros — detalhar depois). Com efeito: ATB 100% para a
  party, inimigos com ATB 0% e **Distraídos**. Sem efeito: batalha normal. Ao
  clicar em "Emboscada" o jogador **nunca** é emboscado.
- **Status Distraído (Distracted)** (vale para os dois lados): cada combatente
  emboscado fica Distraído — avatar virado de costas — até o que acontecer
  primeiro:
  1. levar um golpe — esse golpe causa 150% de dano e ele se vira;
  2. a própria ATB chegar a 100% — ele se vira sozinho, sem ter sido atingido.

  Ex.: num 3v3, dois heróis atacam o inimigo A e o terceiro ataca o B. O
  primeiro golpe em A causa 150% e A se vira, então o segundo já é normal; o golpe
  em B causa 150%. O inimigo C segue Distraído até ser atingido ou até a ATB dele
  encher.
- **Chance de ser emboscado:** base da região/trilha (floresta densa > campo
  aberto), modificada por atributos da party (Dexterity, Luck) e por itens e habilidades
  (detalhar quando chegarmos em itens e skills).

**Estruturas de dados que isso exige:**

- Grafo de peças do mundo (vizinhança por borda).
- Por instância: um ponto de entrada por borda vizinha + ponto fixo de chegada
  por teleporte/montaria.
- Rotas de carruagem (sequência de peças) e grafo de portos (arestas de 1 a 3
  saltos).
- Por trecho/peça: chance de encontro e tabela de monstros com nível.

### Batalha (layout)

```
┌──────────────────────────────────────────────────────────┐
│   [avatar] Lobo                           Kael [avatar]   │
│            ▓▓▓▓▓▓░░                   ▓▓▓▓▓▓▓▓            │
│   [avatar] Lobo         ⚔ ← corte     Lyra [avatar]       │
│   [avatar] Serpente                  Doran [avatar]       │
├──────────────────────┬───────────────────────────────────┤
│ ▶ Atacar             │ Kael   HP 312/400  MP 20  [ATB▓▓▓░] │
│   Habilidade         │ Lyra   HP 180/220  MP 64  [ATB▓▓▓▓] │
│   Item               │ Doran  HP 410/410  MP 12  [ATB▓░░░] │
│   Defender           │                                     │
└──────────────────────┴───────────────────────────────────┘
```

Avatares **não se movem**. Os únicos efeitos são de ataque e habilidade,
desenhados **sobre o avatar alvo**: corte de espada, flecha cravando, chamas,
gelo, raio, brilho de cura, número de dano subindo, avatar piscando/tremendo de
leve.

### Estilo de arte

**Vetor plano (SVG)** em todo o jogo, para nada parecer peça de jogos diferentes:

- **Paleta única** (arquivo de tokens compartilhado) — mapas, avatares, fundos e
  interface usam só ela.
- **Contorno escuro** de espessura fixa por escala; sombreamento em 2–3 tons
  chapados, sem gradiente.
- **Biblioteca de peças** reutilizadas: árvore, rocha, telhado, parede, água,
  ponte; para avatares, rosto, olhos, boca, cabelo, roupa, acessório.
- **Avatares montados por peças** (como um criador de personagem): heróis, NPCs e
  monstros no mesmo traço; variações de cor/elemento e "versão chefe" a partir da
  mesma base.
- **Fundos de diálogo:** ~10–15 reutilizáveis (casa, taverna, loja, guilda,
  caverna, floresta, praia, castelo…).
- Monstros simples e diretos (lobo, slime, morcego, esqueleto, serpente) — a
  gameplay está no mapa, na batalha e nos menus, não na arte.
- Inspiração de clima: mapa de SNES visto de cima, colorido e legível. Mundo e
  geografia próprios — nada copiado de jogos existentes.

---

## 3. Sistema de batalha (ATB)

O tempo da batalha avança em **ticks fixos de 100 ms**. A cada tick, a barra ATB
de cada combatente vivo enche conforme a sua velocidade; quem chega a 100% pode
agir. A ação escolhida entra numa **fila** e executa em ordem de chegada,
enquanto as outras barras continuam enchendo.

### Ciclo de um combatente

```
            ┌──────────────────────────────────────────────────────┐
            ▼                                                      │
   ┌──────────────┐  barra 100%  ┌──────────┐  comando  ┌─────────┐ │ animação
   │  Carregando  │ ───────────▶ │  Pronto  │ ────────▶ │ Na fila │ │ termina,
   │ (barra enche)│              │ (escolhe)│           └────┬────┘ │ barra = 0
   └──────────────┘              └──────────┘                │      │
          ▲                                          vez dele │      │
          │ Sono acaba / revive                              ▼      │
   ┌──────────────┐                                    ┌──────────┐ │
   │  Parado      │ ◀─── Sono, Stop, KO ─────────────  │Executando│─┘
   └──────────────┘                                    └──────────┘
```

Só uma ação executa por vez (a fila é global); as barras não param durante a
animação no modo Ativo.

### Barra ATB

```
ganho por tick = BASE × (Dexterity + 50) / 100 × mod_status
```

- `BASE = 2,5%` → Dexterity 50 enche a barra em **4 s**; Dexterity 100, em
  ~2,7 s. Valores calibrados no simulador para a faixa real de Dexterity (1 a
  255).
- `mod_status`: Haste = 2, Slow = 0,5, Stop/Sono = 0, normal = 1.
- Ao agir, a barra volta a 0.
- Início da batalha: barras começam em valor aleatório de 0–30%.
  **Emboscada** (seção 2): quem embosca começa com ATB 100%; o lado emboscado
  começa com ATB **0%** e **Distraído** — o próximo golpe em cada um causa 150%, até ser atingido ou a
  própria ATB encher. Vale para os dois lados.

### Modos de tempo

- **Ativo:** o tempo nunca para, nem com menu aberto. Único modo do multiplayer.
- **Espera:** o tempo pausa enquanto o jogador navega em submenus (Magia, Item).
  Só no single-player — online, um jogador não pode congelar a party.
- **Velocidade:** multiplicador global de 0,5× a 2× nas opções (só no Modo História).

### Comandos

| Comando | O que faz | Custo |
| --- | --- | --- |
| Atacar | Dano físico num alvo | — |
| Magia | Dano elemental, cura ou status | MP |
| Habilidade | Técnica da classe (Provocar, Roubar, Golpe duplo…) | MP ou cooldown |
| Item | Poção, Éter, Antídoto, Pena de Fênix | Item do inventário |
| Defender | Recebe 50% de dano até a próxima ação | — |
| Limit | Golpe especial quando a barra de Limit está cheia | Barra de Limit |

**Não existe fuga:** toda batalha vai até a vitória ou a derrota.

A barra de **Limit** enche com o dano recebido (`dano / HP máx × 100%`). Cheia, o
comando Atacar vira Limit até ser usado.

### Atributos

**Modelo do FF7**, que já foi desenhado para ATB e turnos. Atributos **não**
entram na Mastery Tree.

| Atributo primário (1–255) | Alimenta |
| --- | --- |
| Strength | Attack = Strength + arma |
| Dexterity | Velocidade da ATB, acerto e esquiva |
| Vitality | Defense = Vitality + armadura |
| Magic | Magic Attack, força de cura |
| Spirit | Magic Defense = Spirit + armadura |
| Luck | Chance de crítico, resistência a status, roubo |

- **HP** (até 9999) e **MP** (até 999) crescem pela curva da classe, não por
  pontos.
- **Heróis da história:** curvas fixas por classe, sem aleatoriedade.
- **Guerreiros recrutados:** pontos para distribuir a cada nível, com **custo
  crescente** (estilo Ragnarok Online): subir um atributo custa mais quanto mais
  alto ele já está, o que impede jogar tudo num atributo só. O total por nível é
  calibrado para que um recruta no nível 99 tenha a mesma soma de atributos que
  um herói.
- **Sources** (itens que dão +1 permanente num atributo, como no FF7): só no
  **late game, depois do nível 99**, e com **limite** por atributo e total,
  calculado para não repetir o "farm até 255 em tudo" do FF7. É uma atividade a
  mais para o fim do jogo.

### Dano

Estrutura do FF7, em aritmética inteira:

```
físico   base = Attack + ⌊(Attack + Nível) / 32⌋ × ⌊(Attack × Nível) / 32⌋
mágico   base = 6 × (Magic Attack + Nível)
dano     = Poder × (512 − Defesa) × base / (16 × 512)
         × variação [3841..4096] / 4096 × elemento × crítico
final    = dano × 150% se o alvo está Distraído   (aplicado por último)
         → limitado a 9999
```

- **Poder:** força da ação (ataque normal = 16; magias e habilidades com valores
  próprios).
- **Defesa** (Defense ou Magic Defense) vai de 0 a 511; 512 seria imunidade, e
  equipamento e status respeitam esse teto.
- **Elemento:** fraqueza 2×, resistência 0,5×, imune 0.
- **Ordem:** o dano é calculado por inteiro (incluindo elemento e crítico) e,
  **por último**, recebe o multiplicador de 150% do Distraído. Os dois se
  acumulam de forma multiplicativa: crítico base 130% × Distraído 150% =
  **195%**. O teto de 9999 vale sobre o resultado final.
- **Crítico:** dano base 130%, aumentado por equipamento (e, no futuro, talvez
  cartas). Chance a partir de Luck, nível do atacante contra o do alvo, e bônus
  da arma; fontes de crítico em equipamento têm teto (sem empilhar como o
  Luck Plus do FF7).
- **Cura:** `Poder × 22 + (Nível + Magic) × 6`, sem defesa.
- **Gravity** (elemento): dano em **% do HP atual** do alvo; chefes resistem ou
  são imunes.
- **Acerto (ataques físicos):**
  `precisão da arma + (Dexterity do atacante − Dexterity do alvo) / 4 − esquiva do alvo`,
  entre **25% e 100%**. Precisão padrão da arma: 95%; esquiva vem de armadura e
  escudo. Alvo **Distraído ou dormindo não esquiva**. O crítico só é sorteado
  depois do acerto; ataques em área sorteiam por alvo.
- **Magias de dano, cura e itens sempre acertam.**
- **Status hostis:** `precisão do status − Luck do alvo / 4`, entre **5% e
  100%**. Imunidade é absoluta e separada. Status benéficos em aliados sempre
  funcionam.
- **Na tela:** **Miss** (ataque físico errou), **Resist** (status falhou pela
  Luck), **Immune** (imunidade a status), **0** (dano totalmente anulado, em
  geral magia contra afinidade imune).
- **Teto de dano: 9999.** Calibrado para que **mesmo com crítico** seja
  praticamente impossível um personagem chegar a 9999 num monstro. Chefes de
  raid e monstros muito fortes **podem** causar 9999.
- **Barrier / Magic Barrier:** reduzem à metade o dano físico / mágico recebido;
  a defesa contra golpes que matariam de uma vez.
- **PvP (futuro):** redução de dano em % entre jogadores, para que 9999 de HP
  faça sentido.
- Mínimo de 1 de dano. **Tudo em inteiros** (porcentagens em pontos-base, ex.:
  2,5% = 250/10000) para a simulação ser determinística.

### Status

Veneno (−5% HP a cada 3 s), Sono (barra para até levar dano), Silêncio (sem
Magia), Haste, Slow, Barrier (dano físico −50%), Magic Barrier (dano mágico
−50%), Regen, **Distraído** (próximo golpe recebido causa 150% de dano; acaba ao ser
atingido ou com a ATB em 100%; aplicado pela emboscada e, no futuro, por
habilidades — ex.: inimigo psíquico).

O efeito visual do Distraído é o avatar virado de costas: como os avatares são
montados por peças, cada um ganha uma variante "de costas" gerada pelas mesmas
peças (nuca, cabelo, roupa).

### IA dos inimigos

Cada inimigo tem uma tabela de regras avaliada quando a barra dele enche:

```ts
// exemplo
[
  { se: "hp < 30%",            faz: "Cura",   alvo: "self" },
  { se: "heroi com Barrier",   faz: "Dispel", alvo: "esse heroi" },
  { peso: 70,                  faz: "Atacar", alvo: "heroi com menos HP" },
  { peso: 30,                  faz: "Fogo",   alvo: "heroi aleatorio" },
]
```

Chefes trocam de tabela por fase de HP (ex.: abaixo de 50% vira mais agressivo).

### Fim de batalha

- **Vitória:** todos os inimigos caem → XP, gil, drops.
- **Derrota:** todos os heróis caem → volta à última sala de descanso.

---

## 4. Conteúdo e progressão

O MVP tem três classes com papéis claros e uma primeira região do mapa (uma
cidade, uma zona de exploração e um calabouço com chefe). Todo o conteúdo vive em arquivos de dados (`content/`), não no
código da engine.

### Heróis e Guerreiros recrutados

- **Heróis da história (Modo História):** personagens fixos, cada um com classe
  e Mastery Tree próprias. Os **atributos evoluem sozinhos**, em ritmo fixo
  conforme a classe; a **ordem das casas da Mastery Tree o jogador escolhe**. Um
  NPC da história pode entrar na party temporariamente (até 4 membros).
- **Guerreiros recrutados (Modo Online):** criados na Guilda de Aventureiros.
  Classe escolhida no recrutamento e fixa depois disso. A cada nível, o jogador
  distribui os pontos de atributo — mesma escala de poder dos heróis, só muda a
  customização.

### Mastery Tree

Sistema de habilidades em grade, inspirado no License Board do FF12, com nome e
desenho próprios.

- **Casas em grade:** libera-se uma casa **vizinha** de outra já liberada,
  gastando 1 ponto de Mastery por casa (custos maiores por casa, se o
  balanceamento pedir).
- **Casas visíveis e desabilitadas** (no FF12 ficavam escondidas), para o
  jogador planejar o caminho.
- **A casa só libera; ainda é preciso obter.** Uma casa de magia ou de
  equipamento dá a **permissão** de usar; a magia ou o item ainda precisam ser
  comprados, dropados ou ganhos. Segura a inflação e impede builds que correm
  atrás da magia mais forte cedo (ex.: Ultima no nível 20): magias de alto nível
  vêm de chefes ou de lojas de cidades avançadas na história.
- **Heróis da história:** uma Mastery Tree própria por personagem.
- **Guerreiros recrutados:** uma Mastery Tree única e grande, compartilhada. A
  classe define **dois pontos de partida**: um na área de equipamentos e outro
  na de magias.
- **Contra builds iguais:**
  - **Teto de pontos** (80 no máximo, ver Progressão): ninguém libera a árvore
    inteira; a escolha é o build.
  - **Algumas áreas exclusivas de classe.**
  - **Pouquíssimos pares exclusivos:** casas em que só faz sentido ter uma; liberar
    uma bloqueia a outra.
- **Reset da árvore inteira:** recrutas, com itens especiais de evento; heróis,
  com um NPC específico.
- Validação no servidor: vizinhança, pontos, exclusividades e classe.

### Classes iniciais (rascunho)

| Classe | Papel | Atributos fortes | Habilidades (nível 1 / 5 / 10) | Limit |
| --- | --- | --- | --- | --- |
| Guerreiro | Tanque e dano físico | HP, Strength, Vitality | Provocar / Golpe duplo / Quebra-armadura | Lâmina Cruzada: 3 golpes num alvo |
| Mago | Dano mágico em área | Magic, MP | Fogo, Gelo, Raio / Fogo em área / Silêncio | Meteoro: dano em todos os inimigos |
| Clérigo | Cura e suporte | Spirit, Magic, Dexterity | Cura, Barrier / Regen, Esuna / Haste, Reviver | Bênção: cura total da party |

**Inimigos (MVP):** ~8 tipos em 3 famílias (bestas, mortos-vivos, elementais),
cada um com uma fraqueza elemental e uma regra de IA marcante, mais 1 chefe com 2
fases.

### Loop do jogo

> **A rediscutir:** o loop abaixo era o da "torre", antes do mapa-múndi. Fica como
> referência até fecharmos as mecânicas.

1. Escolhe a party (3 heróis no single-player).
2. Sobe um andar: 1 a 3 batalhas por andar.
3. Ganha XP, gil e drops.
4. A cada 3 andares, sala de descanso: cura total, loja, troca de equipamento.
5. Andar 10: chefe. Vencer libera o próximo ciclo com inimigos mais fortes.

**Progressão: duas barras de experiência** (como base e class exp do Ragnarok
Online):

| Barra | Nível | Cada nível dá |
| --- | --- | --- |
| Base | 1 a 99 | Atributos (heróis: automático; recrutas: pontos para distribuir) |
| Mastery | 0 a 75 | 1 ponto de Mastery Tree; a cada 15 níveis de Mastery, +1 ponto de bônus |

- Total máximo: 75 + 5 bônus (níveis de Mastery 15, 30, 45, 60 e 75) = **80
  pontos** de Mastery Tree.
- No nível base 99 o personagem para de ganhar experiência de base e passa a
  ganhar só de Mastery, até o Mastery 75.
- Monstros e missões dão experiência de base e de Mastery.
- Meta de balanceamento: ao chegar no nível base 99, o personagem está por volta
  do Mastery 60–63; os últimos pontos vêm depois.
- Equipamento em 3 slots (arma, armadura, acessório), sem crafting no MVP.

### Áreas de caça

Monstros e áreas são desenhados em torno de **três focos**, com troca entre eles
(como as hunts do Tibia): onde se ganha muita experiência costuma dar prejuízo,
e onde se ganha muito loot costuma dar pouca experiência.

| Perfil | Experiência | Loot / dinheiro | Para quê |
| --- | --- | --- | --- |
| Foco em base | Alta (base) | Baixo; gasto com consumíveis maior que o ganho | Subir de nível |
| Foco em Mastery | Alta (Mastery) | Baixo | Pontos de Mastery Tree |
| Foco em loot | Baixa | Alto | Fazer dinheiro |
| Balanceada | Média | Médio | Jogo do dia a dia |
| Perigosa | Ruim | Ruim | Exige atenção; chance pequena de um **item raro** |

- O perfil é definido no conteúdo de cada área de instância e de cada monstro.
- **O jogo não mostra o perfil:** o jogador descobre explorando.
- **Suprimentos são o principal freio da inflação:** o "prejuízo" das áreas de
  experiência vem de consumíveis (poções). Runas podem vir depois, também como
  fonte de renda entre jogadores.
- **Itens raros por drop** (como as cartas do Ragnarok Online, que se encaixam
  em equipamentos): opção para o futuro, se o balanceamento pedir.
- **Munição básica infinita.** Munição especial (fogo, veneno, perfurante) é um
  consumível **opcional**: gasto que o jogador escolhe para ganhar eficiência,
  nunca um imposto obrigatório de uma classe só.

### Experiência em party

A métrica de balanceamento é **experiência por hora**, não por batalha: com a
ATB, 3 personagens terminam uma batalha ~3× mais rápido que 1.

- **Divisão igual por personagem:** a experiência da batalha é dividida entre
  todos os personagens da party, sejam do mesmo jogador ou de jogadores
  diferentes. Por hora, solo e party empatam; a party ganha em segurança e
  suprimentos.
- **Bônus por jogador distinto na party** (ex.: +20% com 2, +40% com 3),
  calibrado com o simulador medindo experiência por hora de cada formação. Só
  conta jogadores que **agiram** na batalha (contra contas secundárias paradas).
- **Faixa de nível para dividir:** no estilo do Ragnarok Online clássico, os
  níveis base do maior e do menor da party precisam estar a até 10 níveis.
- **Penalidade por diferença de nível:** experiência (e drop) caem
  gradualmente quando o personagem está muito acima do nível do monstro. Tabela
  própria, calibrada no simulador. Inspirada no Renewal do Ragnarok Online, a
  curva também **premia** caçar um pouco acima do próprio nível.

### Raridade de itens

| Raridade | Cor |
| --- | --- |
| Common | Cinza |
| Rare | Azul |
| Epic | Roxo |
| Legendary | Laranja |
| Unique | Vermelho |

### Loot em party (Modo Online)

- **Gerado uma vez por batalha**, nunca por jogador (um item raro não se
  multiplica pelo tamanho da party).
- **Direitos por jogador, não por personagem:** quem leva 2 personagens não tem 2
  chances. Experiência é por personagem; loot, por jogador.
- **Faixas:**
  - **Dinheiro:** dividido igualmente entre os jogadores.
  - **Itens abaixo da raridade de leilão:** rodízio/sorteio automático.
  - **Itens a partir da raridade de leilão** (padrão: Rare; configurável pelo
    líder): **leilão** estilo Blade & Soul. Cada jogador dá um lance ou passa;
    o maior lance leva, e o valor é dividido entre os outros membros. Se todos
    passarem, o item é sorteado entre todos.
- O leilão roda num painel lateral com prazo (20–30 s) e **não trava o jogo**.
- **Lance só com dinheiro Tradeable.** O leilão só redistribui dinheiro; não cria
  nem destrói.
- **Modos da party:** leilão (padrão), só sorteio, ou o líder distribui.
- **Configuração:** quem cria a party define modo e raridade de leilão sem
  votação. Quem vai entrar **vê essas configurações antes de aceitar**.
- **Mudança depois:** o líder propõe, só **fora de batalha**; todos são avisados
  e votam. Propor conta como voto a favor do líder; quem não responde no prazo
  conta como **contra**. Maioria aprova; **empate rejeita** (numa party de 2, o
  outro jogador tem, na prática, poder de veto).

**Arte:** toda feita em SVG no estilo da seção 2. **Som:** música e efeitos de
pacotes com licença livre — anotar a licença de cada um em
`apps/web/public/assets/CREDITS.md`.

---

## 5. Arquitetura técnica

Servidor em **Rust**, banco **Postgres**, cliente **SPA em Vite + React +
TypeScript**, tudo em Docker. A engine de regras é um crate Rust puro que roda no
servidor (a verdade) e pode rodar no navegador via **WebAssembly** (só para
prever e suavizar a tela).

| Camada | Escolha | Por quê |
| --- | --- | --- |
| Engine de regras | Rust puro (`crates/engine`), sem I/O | Enums + `match` exaustivo modelam status, comandos e eventos; compila para o servidor e para WASM |
| Servidor | Rust: `tokio` + `axum` | Concorrência sem GC; um ator (task) por sala |
| Banco | Postgres 17 + **SQLx** | Queries verificadas contra o schema em tempo de compilação; migrations versionadas |
| Protocolo | WebSocket com JSON, tipos definidos em Rust e **gerados** para TS (`ts-rs`) | Uma única definição; o TypeScript acusa quando o servidor muda |
| Cliente | Vite + React + TypeScript (SPA, sem SSR) | O jogo vive atrás de login, numa conexão WebSocket; SSR não traria nada |
| Mapas | SVG dentro do React | Nodes e trilhas são elementos clicáveis; sem engine de jogo |
| Efeitos de batalha | CSS (cortes, brilho de magia, números flutuantes); PixiJS só se os efeitos pedirem mais | Leve e sem dependência enquanto CSS bastar |
| Conteúdo | Arquivos JSON validados por JSON Schema gerado do Rust (`schemars`) | Editável sem recompilar regras; autocomplete e validação no editor |
| Traduções | Project Fluent (`.ftl`), `en-US` padrão e `pt-BR` | Plural e gênero tratados; mesma biblioteca existe em Rust e JS |

### Estrutura de pastas

```
rpg/
├── Cargo.toml              workspace Rust + [workspace.lints]
├── crates/
│   ├── engine/             regras puras: ATB, combate, status, IA, viagem, encontros
│   ├── content/            tipos do conteúdo, carga e validação (+ binário content-check)
│   ├── protocol/           mensagens cliente↔servidor (gera os tipos TS)
│   ├── server/             axum, salas, contas, persistência (SQLx)
│   └── sim/                CLI: roda batalhas por seed, simula balanceamento
├── content/                dados do jogo (JSON): classes, monstros, itens, mapas…
├── locales/
│   ├── en-US/*.ftl         textos do jogo (padrão)
│   └── pt-BR/*.ftl         tradução
├── migrations/             SQL versionado (SQLx)
├── web/                    Vite + React + TS
│   └── src/protocol/generated/   tipos gerados — nunca editados à mão
├── docs/decisions/         um arquivo curto por decisão (ADR)
├── Dockerfile              multi-stage: build Rust + build web → imagem distroless
├── docker-compose.yml      produção: rpg_app + rpg_db
├── docker-compose.dev.yml  desenvolvimento local
├── Makefile                up / up-d / down / logs / build / check / dev-*
├── CLAUDE.md               mapa do projeto e regras (criado na fase 0)
└── PLAN.md                 este arquivo
```

Direção das dependências — imposta pelo Cargo, um import fora dela não compila:

```
protocol ──▶ engine ◀── content
    ▲          ▲           ▲
    └──────  server  ──────┘          sim ──▶ engine, content
```

`engine` não conhece banco, rede nem relógio.

### Engine

- **Determinística:** mesma seed + mesmos comandos = mesma batalha. PRNG com seed
  e versão fixada, só aritmética inteira (porcentagens em pontos-base), nenhum
  acesso a relógio ou aleatoriedade do sistema.
- **Passo fixo:** `step()` avança exatamente 1 tick (100 ms). O servidor chama a
  10 Hz.
- **Saída em eventos:** cada passo devolve eventos tipados (`DamageDealt`,
  `StatusApplied`, `Knockout`, `BattleWon`…). O cliente só anima eventos, nunca
  calcula regra.
- **Erros de regra são valores:** um comando inválido devolve um erro tipado
  (`CommandError::AtbNotFull` etc.), nunca pânico.

### Servidor

- **Um ator por sala:** cada sessão do Modo História e cada party online é uma
  task `tokio` dona do próprio estado em memória, recebendo comandos por canal.
  Não há lock compartilhado no caminho do jogo.
- **Banco fora do caminho quente:** a batalha nunca espera o disco. O estado é
  gravado em checkpoints (fim de batalha, mudança de node, compra, troca), e
  trocas entre jogadores são uma transação Postgres.
- **HTTP:** `/api/health`, autenticação e o build estático do `web/`.
- **WebSocket:** `/ws`, com reconexão por token de sessão.
- **Logs:** `tracing`, em JSON no container.
- **Config:** variáveis de ambiente lidas uma vez na inicialização, num struct
  tipado.

### Cliente

- React para telas, janelas, HUD e menus; SVG para os mapas; CSS para os
  efeitos de batalha (PixiJS só se os efeitos pedirem mais).
- Componentes de UI reutilizáveis em `web/src/ui/` (janela, lista de menu,
  barra, botão, select…); as telas só compõem esses componentes, sem repetir
  marcação.
- O estado de jogo vem do servidor; o cliente guarda só preferências (idioma,
  volume, velocidade de texto).
- Barras ATB interpoladas entre ticks para ficarem lisas a 60 fps.
- A engine em WASM entra só se a previsão local se provar necessária; até lá, o
  cliente apenas anima os eventos do servidor.

### Banco de dados

- Postgres 17 na rede interna do compose — **nunca** na `proxy_net`.
- Migrations SQLx em `migrations/`, aplicadas na inicialização do servidor.
- Inventário com a tag de origem (`untradeable`, `tradeable`, `temporary`) como
  enum do Postgres.
- **Backup desde o primeiro dia:** `pg_dump` diário, retenção de 7 diários + 4
  semanais, fora do volume do banco.

### Traduções (i18n)

- O jogo nasce em **inglês (`en-US`, padrão)**, com **português (`pt-BR`)**
  escolhível nas configurações.
- **Todo texto mostrado ao jogador é uma chave**, nunca uma string no código. O
  servidor manda chaves + parâmetros; o cliente renderiza no idioma escolhido.
- Formato **Project Fluent** (`.ftl`): resolve plural ("1 potion" / "3
  potions") e concordância, que um simples `{chave: texto}` não resolve.
- Chaves derivadas do id do conteúdo: o item `iron-sword` tem
  `item-iron-sword` (nome) e `.description` (descrição).
- `make check` falha se uma chave existe num idioma e não no outro, ou se o
  conteúdo cita uma chave inexistente.
- Chat não é traduzido.

---

## 6. Padrões e guarda-corpos

Objetivo: o projeto tem **um lugar para cada coisa**, e o que sobra de decisões
antigas é **detectado pela máquina**, não pela memória de quem programa.

### Idioma e nomes

- Código, identificadores, ids de conteúdo, comentários, commits e documentação
  técnica: **inglês**. Este `PLAN.md` é a exceção: documento de conversa, em
  português.
- Nomes do jogo (classes, status, itens) também em inglês, traduzidos via
  `locales/pt-BR`.

| Onde | Convenção | Exemplo |
| --- | --- | --- |
| Rust: tipos, traits, variantes de enum | `UpperCamelCase` | `Combatant`, `Status::Distracted` |
| Rust: funções, variáveis, módulos, arquivos | `snake_case` | `apply_damage`, `atb_gauge.rs` |
| Rust: constantes | `SCREAMING_SNAKE_CASE` | `TICK_MS` |
| TS: variáveis, funções | `camelCase` | `selectTarget` |
| TS: componentes React, tipos | `PascalCase` (componente em `PascalCase.tsx`) | `BattleScreen.tsx` |
| TS: demais arquivos | `kebab-case` | `use-socket.ts` |
| Ids de conteúdo | `kebab-case`, únicos por tipo | `iron-sword`, `forest-wolf` |
| Chaves de tradução (Fluent) | `<tipo>-<id>` + atributos | `item-iron-sword`, `.description` |
| Tabelas e colunas Postgres | `snake_case`, tabelas no plural | `recruits`, `created_at` |
| Mensagens do protocolo (`type`) | `snake_case` | `battle_command` |
| Commits | Conventional Commits | `feat(engine): add distracted status` |

### Boas práticas (sem overengineering)

- **Newtypes para ids** (`HeroId`, `NodeId`, `ItemId`): impede trocar um id por
  outro.
- **Estados ilegais irrepresentáveis:** enums com dados em vez de combinações de
  booleanos e `Option`s.
- **Erros:** `thiserror` para erros de domínio, `anyhow` só nos binários; sem
  `unwrap()`/`expect()` fora de testes e inicialização.
- **Abstração só com o segundo uso real:** nada de trait, genérico ou camada
  "para o futuro". Duas implementações concretas justificam uma abstração; uma
  não.
- **`unsafe`: evitar ao máximo.** O workspace declara `unsafe_code = "deny"`.
  Se um caso realmente exigir (ex.: uma FFI ou uma otimização medida que não
  tem alternativa segura), só entra com:
  1. `#[allow(unsafe_code)]` no menor escopo possível (um bloco, nunca um
     módulo inteiro);
  2. comentário `// SAFETY:` explicando quais invariantes garantem que aquilo é
     correto (o lint `clippy::undocumented_unsafe_blocks` barra bloco sem ele);
  3. um ADR em `docs/decisions/` explicando por que foi necessário, quais
     alternativas seguras foram tentadas e por que falharam;
  4. o `unsafe` encapsulado atrás de uma função segura, com testes (e Miri,
     quando aplicável).
- TS em modo estrito, com `noUncheckedIndexedAccess`.

### Um lugar para cada coisa

- O `CLAUDE.md` do projeto mapeia onde mora cada conceito (ex.: "status de
  batalha: `crates/engine/src/status.rs`") e é atualizado na mesma mudança que
  cria ou move um conceito.
- Antes de criar arquivo, função ou tipo novo, procurar se já existe.

### Comentários

- Comentário explica **por quê**, nunca o quê, nem o histórico ("antes era X").
- Histórico de decisões vai para `docs/decisions/`; o design atual, para o
  `PLAN.md`.
- Exemplos em doc comments do Rust são compilados e testados (doctests).

### Documentação gerada (rustdoc)

- Itens públicos documentados com `///` e cada módulo com `//!` (o que é, por
  que existe, como se encaixa). O lint `missing_docs` cobra isso em todo o
  workspace.
- Links entre itens no formato `` [`Combatant`] ``: um link quebrado (item
  renomeado ou removido) falha no `make check`.
- `make docs` gera e abre a documentação do projeto inteiro, incluindo itens
  privados, como um site navegável e pesquisável.

### Testes

- Testes unitários no mesmo arquivo do código (`#[cfg(test)] mod tests`): apagou
  o código, o teste vai junto.
- Testes de engine por **replay**: seed + comandos → eventos esperados.
- Teste existe para um comportamento atual. Comportamento removido = teste
  removido. Teste ignorado ou pulado não passa no `make check`.

### Mudança de decisão

Quando uma decisão muda, **a mesma alteração** remove código, testes, conteúdo,
chaves de tradução e comentários antigos, registra um ADR em `docs/decisions/` e
atualiza o `PLAN.md` e o `CLAUDE.md`.

### `make check` — tudo precisa passar

| Alvo | Ferramentas |
| --- | --- |
| Rust | `cargo fmt --check`, `cargo clippy -- -D warnings` (código morto vira erro), `cargo test`, `cargo machete` (dependências não usadas) |
| Frontend | `tsc` estrito, oxlint, Prettier, **knip** (arquivos, exports e dependências sem uso), paridade de chaves entre `en-US` e `pt-BR` |
| Protocolo | Os tipos TS gerados estão atualizados (gerar de novo não muda nada) |
| Conteúdo | `content-check`: schema, referências entre ids, chaves de tradução nos dois idiomas |
| SQL | `cargo sqlx prepare --check` (entra junto com a primeira query) |
| Testes pulados | Nenhum `#[ignore]`, `.skip`, `.only` ou `.todo` |
| Documentação | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items` (links quebrados e docs faltando viram erro) |

Um hook do Claude Code (`.claude/settings.json`) roda o `make check` ao fim de
cada tarefa e não deixa encerrar com ele falhando. Vale para sessões do Claude
Code abertas **dentro de `rpg/`**.

### Ambientes de teste (Sandbox)

Cenários isolados para testar uma coisa de cada vez — como as salas vazias de
teste de combate de estúdios de jogos —, sem passar pela história nem afetar o
jogo.

- **Mesmo código do jogo:** um cenário só define o estado inicial; engine,
  servidor e tela são os de verdade. O que funciona no sandbox funciona no jogo.
- **Cenários como dados**, em `sandbox/` (fora de `content/`, nunca referenciados
  pelo jogo):

  | Cenário (exemplo) | Testa |
  | --- | --- |
  | `battle-basic` | 3v3 simples: ATB, comandos, dano |
  | `ambush-player` / `ambush-enemy` | Emboscadas e o status Distracted |
  | `atb-speeds` | Combatentes com Dexterity muito diferente, Haste/Slow |
  | `status-showcase` | Cada status aplicado de uma vez |
  | `boss-phases` | Troca de tabela de IA por fase de HP |
  | `map-fog` | Instância pequena com fog of war e trilhas bloqueadas |
  | `travel-mount` | Viagem de montaria, desmonte em peça oculta |

- **Cada cenário fixa:** party, inimigos, seed, ATB e status iniciais, e opções
  de teste (party invencível, MP infinito, velocidade do tempo).
- **Ferramentas de debug na tela:** pausar e avançar tick a tick, painel com ATB
  exata, log de eventos, seed atual, aplicar status, derrubar combatente.
- **Acesso:** `localhost:8083/sandbox/<cenário>?seed=42`.
- **Fora da produção por construção:** tudo do sandbox fica atrás da feature
  `sandbox` do Cargo e de uma flag no build do `web/`. A imagem de produção é
  compilada sem ela: o código nem existe lá.
- **Testes e2e:** Playwright abre os cenários com seed fixa e verifica a tela; os
  testes da engine carregam os mesmos arquivos de cenário. `make e2e` roda à
  parte (é mais lento que o `make check`).
- O `content-check` também valida os cenários (ids e referências).

---

## 7. Multiplayer

É o **Modo Online**: party de no máximo 3, formada por Guerreiros recrutados
(nunca heróis da história), com o **servidor como única fonte da verdade**. O
cliente só manda intenções ("Guerreiro X usa Fogo no inimigo 2") e desenha o que
o servidor responde.

### Party e controle

- Party padrão: **3 Guerreiros**. Calabouços e raids específicos podem aumentar
  o limite para 4, 6 ou 8.
- O **líder** decide quantos Guerreiros cada jogador controla (mínimo 1 por
  jogador).
- Cada jogador escolhe as ações dos **seus** Guerreiros quando a ATB deles enche.
- O líder decide o caminho no mapa-múndi e nas instâncias, e conduz os diálogos
  com NPCs.
- A party só pode entrar em regiões que **todos** os membros já liberaram no Modo
  História (é o que equilibra um modo com o outro).
- Se o líder sai ou desconecta, a liderança passa para o próximo na ordem de
  entrada (o segundo, depois o terceiro, …).
- Em lojas, cada jogador age de forma independente, dentro da própria janela da
  loja.

### Chat

Só no Modo Online (desligado no Modo História): canal **global**, canal da
**party**, **mensagem privada** e até **3 canais customizados** por jogador.

### Formar party

1. Um jogador cria a party → recebe um código de 4 letras e um link.
2. Os outros entram pelo link e escolhem quais Guerreiros recrutados vão levar.
3. O líder distribui o controle e parte.

### Fluxo de uma ação

```
 jogador B                    servidor (10 Hz)                     todos
    │  battle_command ──────▶ valida ─▶ fila do tick seguinte
    │                         step() ─▶ eventos ─────────────────▶ animam
    │                         a cada 1 s: snapshot ──────────────▶ corrigem desvio
```

### Mensagens (`crates/protocol`)

| Direção | `type` | Conteúdo |
| --- | --- | --- |
| C → S | `party_create` / `party_join` | código |
| C → S | `party_assign` | Guerreiro → jogador (só o líder) |
| C → S | `battle_command` | Guerreiro, comando, alvos |
| S → C | `party_state` | jogadores, Guerreiros, líder |
| S → C | `battle_tick` | nº do tick + eventos daquele tick |
| S → C | `battle_snapshot` | estado completo (a cada 1 s e ao reconectar) |
| S → C | `battle_ended` | resultado, recompensas |

### Regras de validação (anti-trapaça)

O servidor recusa a ação se: o Guerreiro não é controlado pelo jogador, a ATB não
está cheia, falta MP/item, o alvo é inválido ou está KO, ou o Guerreiro está
silenciado/dormindo. O cliente nunca manda dano, HP ou resultado.

### Reconexão e latência

- Reconectando em até **60 s**, o jogador retoma os mesmos Guerreiros e recebe um
  snapshot. Enquanto fora, eles usam Defender quando a ATB enche.
- Passado o prazo, o controle volta para o líder.
- Tick de 100 ms tolera bem 100–200 ms de ping num jogo de menu; a ação entra no
  tick em que chega ao servidor.
- Modo **Ativo obrigatório** online; não há pausa.
- A latência vem da distância física, não da linguagem. Servidores em outras
  regiões ficam para quando houver jogadores longe que justifiquem.

---

## 8. Infra no my-server

O `rpg/` vive como pasta irmã dos outros serviços, **com repositório git
próprio** — o my-server só guarda o server block do nginx, como já é feito com o
giveaway.

| Item | Valor |
| --- | --- |
| Pasta | `my-server/rpg/` (git próprio) |
| Containers | `rpg_app` (servidor + estáticos), `rpg_db` (Postgres) |
| Porta interna | 3000 (`rpg_app`) |
| Porta local | `127.0.0.1:8083` |
| Redes | `rpg_app`: `default` + `proxy_net`; `rpg_db`: só `default` |
| Volumes | `pgdata` (banco), backups fora dele |
| Subdomínio | `rpg.shiraori-games.com` |

### Mudanças no repositório do my-server

1. `.gitignore`: adicionar `/rpg/`.
2. `nginx/conf.d/13-rpg.conf`:

```nginx
# rpg.shiraori-games.com  →  rpg_app (axum: HTTP + WebSocket no mesmo processo)
server {
    listen 80;
    server_name rpg.shiraori-games.com;

    include /etc/nginx/snippets/comum.conf;

    location / {
        set $alvo rpg_app:3000;
        proxy_pass http://$alvo$request_uri;
        include /etc/nginx/snippets/proxy.conf;
        include /etc/nginx/snippets/timeouts-padrao.conf;
    }

    # WebSocket do jogo: conexão longa. Timeouts próprios, por isso SEM timeouts-padrao.conf.
    location /ws {
        set $alvo rpg_app:3000;
        proxy_pass http://$alvo$request_uri;
        include /etc/nginx/snippets/proxy.conf;
        proxy_connect_timeout 10s;
        proxy_send_timeout 86400s;
        proxy_read_timeout 86400s;
    }
}
```

3. `Makefile`: incluir `rpg` na lista do alvo `rota`.
4. `README.md`: linha na árvore de pastas e na tabela de serviços
   (`rpg` · `127.0.0.1:8083` · `rpg_app` · `rpg`).
5. `make reload` (valida antes de aplicar).

### O que fica com você

- **Cloudflare:** Zero Trust → Networks → Tunnels → shiraori → Routes → Add
  route: hostname `rpg.shiraori-games.com`, Service `http://nginx_proxy:80`.
- Conferir com `make rota` (local) e depois abrindo o subdomínio.

### docker-compose.yml do rpg (esboço)

```yaml
services:
  app:
    build: .
    container_name: rpg_app            # contrato com 13-rpg.conf
    restart: unless-stopped
    env_file: .env
    environment:
      PORT: "3000"                     # contrato com 13-rpg.conf
      DATABASE_URL: postgres://rpg:${POSTGRES_PASSWORD}@db:5432/rpg
    ports:
      - "127.0.0.1:${LOCAL_PORT:-8083}:3000"
    depends_on:
      db:
        condition: service_healthy
    networks:
      - default
      - proxy_net
    healthcheck:
      # imagem distroless não tem shell nem curl: o próprio binário checa
      test: ["CMD", "/app/rpg-server", "healthcheck"]

  db:
    image: postgres:17-alpine
    container_name: rpg_db
    restart: unless-stopped
    environment:
      POSTGRES_USER: rpg
      POSTGRES_DB: rpg
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - pgdata:/var/lib/postgresql/data
    networks:
      - default                        # nunca na proxy_net
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rpg"]

volumes:
  pgdata:

networks:
  proxy_net:
    external: true
```

Na nuvem, as mesmas imagens sobem em qualquer provedor, com o Postgres virando um
serviço gerenciado.

---

## 9. Roadmap

```
 Fase 0        Fase 1         Fase 2          Fase 3          Fase 4          Fase 5
 Esqueleto ─◆─ Engine     ─◆─ Cliente +   ─◆─ Conteúdo    ─◆─ Multiplayer ─◆─ Polimento
 e padrões     headless       servidor         e progressão    (Modo Online)
```

| Fase | Entrega | Critério de pronto (◆) |
| --- | --- | --- |
| 0. Esqueleto e padrões ✅ | Git local, workspace Rust, `web/`, Dockerfile, compose (app + db), Makefile, `make check` completo, `make docs`, `CLAUDE.md`, hook, backup do banco | `/api/health` respondendo em `localhost:8083`; `make check` verde |
| 1. Engine headless ✅ | ATB, fila, comandos, fórmulas, status (incl. Distracted), emboscadas, IA por regras, PRNG com seed, conteúdo provisório em `content/`, primeiro cenário `sandbox/battle-basic.json` | Testes de replay passando; `make sim` roda uma batalha 3×3 até o fim e dá o mesmo resultado sempre |
| 2. Cliente + servidor ✅ | Sala de um jogador, tela de batalha, barras, menus, alvos, efeitos básicos, modos Active/Wait, i18n `en-US`/`pt-BR`, sandbox com `battle-basic` e ferramentas de debug, primeiro teste e2e | `localhost:8083/sandbox/battle-basic` jogável do início ao fim, nos dois idiomas; e2e verde |
| 3. Conteúdo e progressão | Mapa-múndi, 1ª cidade, 1 zona de exploração, 1 calabouço com chefe, diálogos, fog of war, ~8 inimigos, XP/nível, loja, save | Região jogável do início ao chefe; save sobrevive a recarregar a página |
| 4. Multiplayer | Guilda e Guerreiros recrutados, party, controle por jogador, validação, snapshots, reconexão | 3 navegadores em redes diferentes vencem uma batalha juntos via subdomínio; derrubar o Wi-Fi de um por 30 s e ele volta aos mesmos Guerreiros |
| 5. Polimento | Chat e moderação, trocas, balanceamento, sons, mais conteúdo | A definir ao fim da fase 4 |

Publicação em `rpg.shiraori-games.com` (server block, `.gitignore` do my-server,
rota no túnel) só quando você decidir; até lá, tudo roda em `localhost`. Contas
entram quando o jogo sair do sandbox para o Modo História.

---

## 10. Riscos e decisões em aberto

### Riscos

| Risco | Mitigação |
| --- | --- |
| Escopo crescer antes do combate ser divertido | Fases 1–2 só com 1 classe e 2 inimigos; conteúdo só na fase 3 |
| Código duplicado, decisões antigas sobrevivendo, comentários e testes desatualizados | Seção 6: `CLAUDE.md`, `make check` com detecção de código morto, protocolo de mudança de decisão |
| Engine acoplada a rede/banco | Direção de dependências entre crates imposta pelo Cargo |
| Simulação divergir entre cliente e servidor | Só inteiros, PRNG com seed, servidor manda snapshot a cada 1 s |
| Balanceamento demorado | `sim` roda 1000 batalhas e imprime taxa de vitória por party |
| Montar 40–80 instâncias à mão (8–20 nodes cada) | Editor interno (página só de dev) para posicionar nodes, trilhas e bloqueios sobre a geografia |
| Arte inconsistente entre telas | Paleta e biblioteca de peças compartilhadas; nenhum asset fora delas |
| Tradução esquecida | `make check` falha com chave faltando em qualquer idioma |
| Perda de dados de jogadores | Backup diário do Postgres desde a fase 0 |
| Licença de sons | `CREDITS.md` desde o primeiro arquivo |
| Save da história forjado para liberar regiões e itens no Modo Online | Simulação e save da história no servidor (decidido) |
| Chat global sem moderação | Limite de mensagens por minuto, silenciar/bloquear jogador, denúncia, filtro de palavras |

### Decisões em aberto

- [ ] Penalidade por morte (XP, dinheiro, itens?) — decidir mais tarde.
- [ ] Modo Online: o que move o jogo (missões da Guilda, exploração livre, chefes, eventos, outros) — detalhar no futuro.
- [ ] Itens raros encaixáveis em equipamento (estilo cartas do Ragnarok Online): decidir depois do balanceamento.
- [ ] Números de atributos (curvas por classe, pontos por nível dos recrutas, tabela de custo, limites de Sources, Poder de cada ação, teto de chance e de dano de crítico): calibrar no simulador.
- [ ] Design visual da Mastery Tree (proposta minha, ajustada jogando).
- [ ] Itens, habilidades e atributos que alteram as chances de emboscada (dos dois lados) — detalhar junto com itens e skills.
- [ ] Nomes em inglês de classes, status, comandos e itens — definir junto com o conteúdo.
- [ ] Nome do jogo.
- [ ] Contas (necessárias desde o Modo História): login próprio, magic link por e-mail, login com Google/Discord?
