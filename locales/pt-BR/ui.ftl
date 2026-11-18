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
ui-command-limit = Limit
ui-command-magic = Magia
ui-command-ability = Habilidade
ui-command-item = Item
ui-command-defend = Defender
ui-command-back = Voltar
ui-command-choose-target = Escolha um alvo
ui-command-waiting = Aguardando um turno…
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
ui-log-limit = { $actor }: Limit pronto
ui-log-knocked-out = { $target } foi nocauteado
ui-log-failed = { $actor } falha em { $action }
ui-log-cancelled = { $actor } cancela { $action }
