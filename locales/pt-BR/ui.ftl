# Textos da interface (menus, configurações, status). O conteúdo do jogo fica em arquivos próprios.

ui-game-title = RPG

ui-server-status = { $status ->
    [ok] Servidor online
    [checking] Verificando o servidor…
   *[unavailable] Servidor indisponível
}

ui-language = Idioma
ui-language-en-US = English
ui-language-pt-BR = Português

## Sandbox

ui-sandbox-title = Sandbox
ui-sandbox-scenarios = Cenários do sandbox

## Batalha

ui-battle-connecting = Conectando…
ui-battle-disconnected = Conexão perdida. Recarregue a página.
ui-command-attack = Atacar
ui-command-magic = Magia
ui-command-technic = Técnica
ui-command-item = Item
ui-command-back = Voltar
ui-command-choose-target = Escolha um alvo
ui-command-waiting = Aguardando um turno…
ui-battle-keys = ↑↓ escolher · Enter/Z confirmar (segurar: repetir a última) · Esc/X voltar · ← defender · Tab próximo herói · M memória
ui-battle-sound = Som: { $state ->
    [on] ligado
   *[off] desligado
}
ui-battle-memory = Memória de ação: { $state ->
    [on] ligada
   *[off] desligada
}
ui-intent-defend = Defender
ui-stat-hp = HP
ui-stat-mp = MP
ui-mp-cost = { $cost } MP
ui-item-count = ×{ $count }
ui-popup-miss = Miss
ui-popup-resist = Resist
ui-popup-immune = Immune
ui-outcome-victory = Vitória!
ui-outcome-defeat = Derrota…
ui-outcome-restart = Lutar de novo

## Status

ui-status-poison = Veneno
ui-status-regen = Regen
ui-status-sleep = Sono
ui-status-silence = Silêncio
ui-status-haste = Aceleração
ui-status-slow = Lentidão
ui-status-barrier = Barreira
ui-status-magic_barrier = Barreira Mágica
ui-status-distracted = Distraído

## Comandos recusados e erros

ui-reject-no-battle = Não há batalha em andamento.
ui-reject-not-player-controlled = Esse personagem não é seu.
ui-reject-not-awaiting-command = Esse personagem ainda não está pronto.
ui-reject-unknown-action = Esse personagem não conhece essa ação.
ui-reject-silenced = Silenciado: sem magia.
ui-reject-not-enough-mp = MP insuficiente.
ui-reject-out-of-item = Não sobrou nenhum.
ui-reject-invalid-target = Alvo inválido.
ui-error-bad-message = O servidor não entendeu o pedido.
ui-error-sandbox-disabled = O sandbox não está disponível neste servidor.
ui-error-unknown-scenario = Cenário não encontrado.
ui-error-invalid-speed = Velocidade não permitida.

## Painel de debug

ui-debug-title = Debug
ui-debug-tick = Tick { $tick }
ui-debug-seed = Seed
ui-debug-pause = Pausar
ui-debug-resume = Continuar
ui-debug-step = Avançar
ui-debug-speed = Velocidade
ui-debug-time-mode = Tempo
ui-time-active = Ativo
ui-time-wait = Espera
ui-time-turn = Por turno
ui-debug-auto = Batalha automática
ui-debug-restart = Reiniciar
ui-debug-fill = ATB
ui-debug-knock-out = KO
ui-debug-apply-status = Status
ui-debug-status = Status a aplicar
ui-debug-log = Log de eventos

## Log de eventos

ui-log-action = { $actor } usa { $action } em { $targets }
ui-log-damage = { $target } sofre { $amount }{ $tags }
ui-log-tag-critical = crítico
ui-log-tag-distracted = distraído
ui-log-tag-weak = fraqueza
ui-log-tag-resist = resistiu
ui-log-tag-immune = imune
ui-log-heal = { $target } recupera { $amount } de HP
ui-log-mp = { $target } recupera { $amount } de MP
ui-log-revive = { $target } revive com { $hp } de HP
ui-log-poison = { $target } perde { $amount } de HP pelo veneno
ui-log-regen = { $target } regenera { $amount } de HP
ui-log-miss = Miss em { $target }
ui-log-status-gain = { $target } recebe { $status }
ui-log-status-lose = { $target } perde { $status }
ui-log-resist = { $target } resiste a { $status }
ui-log-immune = { $target } é imune a { $status }
ui-log-defend = { $actor } defende
ui-log-knocked-out = { $target } foi nocauteado
ui-log-failed = { $actor } falha em { $action }
ui-log-cancelled = { $actor } cancela { $action }

## Conta

ui-account-sign-up = Criar conta
ui-account-log-in = Entrar
ui-account-log-out = Sair
ui-account-recover = Trocar a senha com a recovery key
ui-account-forgot = Esqueceu a senha?
ui-account-have-account = Já tem conta? Entrar
ui-account-logged-in = Conectado como { $username }
ui-account-username = Usuário
ui-account-username-rule = 8 a 20 letras e números, sem espaços nem símbolos.
ui-account-password-current = Senha
ui-account-password-new = Nova senha
ui-account-password-confirm = Repita a senha
ui-account-password-rule = 8 a 64 caracteres.
ui-account-recovery-key = Recovery key
ui-account-error-mismatch = As duas senhas são diferentes.
ui-account-error-invalid_username = O usuário precisa de 8 a 20 letras e números, nada mais.
ui-account-error-invalid_password = A senha precisa de 8 a 64 caracteres.
ui-account-error-username_taken = Esse usuário já existe.
ui-account-error-wrong_credentials = Usuário ou senha incorretos.
ui-account-error-wrong_recovery_key = Usuário ou recovery key incorretos.
ui-account-error-too_many_attempts = Tentativas demais. Espere 15 minutos e tente de novo.
ui-account-error-not_logged_in = Você não está conectado.
ui-account-error-unavailable = O servidor está indisponível. Tente mais tarde.
ui-recovery-key-title = Sua recovery key
ui-recovery-key-explain = Essa chave é o único jeito de trocar a senha se você esquecê-la. Ela só aparece agora.
ui-recovery-key-copy = Copiar
ui-recovery-key-copied = Copiada
ui-recovery-key-warning = Guarde em lugar seguro (um gerenciador de senhas, um papel). Sem ela, esquecer a senha significa perder a conta. Nunca a compartilhe.
ui-recovery-key-saved = Guardei minha recovery key
ui-recovery-key-continue = Continuar
ui-error-not-logged-in = Entre na sua conta para jogar.
ui-error-mode-locked = A história ainda não liberou este modo.
ui-error-no-game = Nenhum jogo aberto.
ui-error-unavailable = O servidor não conseguiu carregar ou salvar o jogo. Tente mais tarde.
ui-error-opened-elsewhere = O jogo foi aberto em outra aba ou aparelho. Recarregue para jogar aqui.
ui-error-too-many-messages = Mensagens demais: vá com calma.

## Jogo

ui-play = Jogar
ui-mode-campaign = Campanha
ui-mode-multiplayer = Multiplayer
ui-mode-new = Novo jogo
ui-mode-continue = Continuar
ui-mode-locked = Bloqueado
ui-mode-locked-hint = O Multiplayer é liberado mais adiante na história da Campanha.
ui-crowns = { $amount } coroas
ui-continue = Continuar
ui-close = Fechar
ui-world-map = Mapa-múndi
ui-time-mode = Tempo de batalha
ui-map-keys = Setas andam · Enter/Z usa · Esc party · M mapa-múndi
ui-node-exit = Ir para { $place }
ui-node-battle = Lutar
ui-node-boss = Enfrentar o chefe
ui-node-ambush = Emboscar
ui-node-chest = Abrir o baú
ui-node-npc = Falar com { $name }
ui-node-shop = Comprar
ui-node-temple = Descansar e salvar aqui
ui-shop-title = Loja
ui-shop-buy = Comprar
ui-shop-sell = Vender
ui-shop-nothing = Nada para vender.
ui-shop-line = { $price } · tem { $owned }
ui-shop-keys = ↑↓ escolhe · Enter/Z compra ou vende um · ←/→ comprar ou vender · Esc sai
ui-shop-leave = Sair
ui-victory-title = Vitória!
ui-victory-xp = { $xp } de experiência
ui-victory-crowns = { $amount } coroas
ui-found-item = { $name } ×{ $count }
ui-level-up = { $name } chegou ao nível { $level }!
ui-learned = Aprendeu { $action }
ui-defeat-title = Derrota…
ui-defeat-text = A party acorda no último templo, curada.
ui-rested-title = Descanso
ui-rested-text = A party está curada. Se cair, volta para cá.
ui-found-title = Encontrado
ui-party-title = Party
ui-party-level = Nível { $level }
ui-party-xp = { $xp } / { $next } de experiência
ui-party-weapon = Arma
ui-party-armor = Armadura
ui-party-none = (nenhuma)
ui-party-no-actions = Nenhuma técnica ou magia ainda
ui-party-use = { $item } ({ $count })
ui-stat-attack = Ataque
ui-stat-defense = Defesa
ui-stat-magic-attack = Magia
ui-stat-magic-defense = Def. Mágica
ui-stat-dexterity = Destreza
ui-stat-luck = Sorte
ui-game-error-busy = Agora não.
ui-game-error-no_such_node = Não dá para ir até lá.
ui-game-error-no_trail = Não há caminho para lá.
ui-game-error-trail_closed = Esse caminho está bloqueado por enquanto.
ui-game-error-barrier = A Campanha ainda não abriu este caminho.
ui-game-error-nothing_here = Nada para fazer aqui.
ui-game-error-no_such_choice = Essa resposta não existe.
ui-game-error-not_for_sale = A loja não negocia isso.
ui-game-error-not_enough_crowns = Coroas insuficientes.
ui-game-error-not_owned = Você não tem isso.
ui-game-error-cannot_equip = Esse herói não pode usar isso.
ui-game-error-not_usable_here = Isso não faz nada agora.
ui-game-error-no_such_hero = Herói inexistente.
ui-game-error-bag_full = Não cabe mais disso.
