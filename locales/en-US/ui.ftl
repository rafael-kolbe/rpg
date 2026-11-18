# Interface text (menus, settings, status). Game content lives in its own files.

ui-game-title = RPG

ui-server-status = { $status ->
    [ok] Server online
    [checking] Checking server…
   *[unavailable] Server unavailable
}

ui-language = Language
ui-language-en-US = English
ui-language-pt-BR = Português

## Sandbox

ui-sandbox-title = Sandbox
ui-sandbox-scenarios = Sandbox scenarios

## Battle

ui-battle-connecting = Connecting…
ui-battle-disconnected = Connection lost. Reload the page.
ui-command-attack = Attack
ui-command-limit = Limit
ui-command-magic = Magic
ui-command-ability = Ability
ui-command-item = Item
ui-command-defend = Defend
ui-command-back = Back
ui-command-choose-target = Choose a target
ui-command-waiting = Waiting for a turn…
ui-stat-hp = HP
ui-stat-mp = MP
ui-mp-cost = { $cost } MP
ui-item-count = ×{ $count }
ui-popup-miss = Miss
ui-popup-resist = Resist
ui-popup-immune = Immune
ui-outcome-victory = Victory!
ui-outcome-defeat = Defeat…
ui-outcome-restart = Fight again

## Statuses

ui-status-poison = Poison
ui-status-regen = Regen
ui-status-sleep = Sleep
ui-status-silence = Silence
ui-status-haste = Haste
ui-status-slow = Slow
ui-status-barrier = Barrier
ui-status-magic_barrier = Magic Barrier
ui-status-distracted = Distracted

## Refused commands and errors

ui-reject-no-battle = There is no battle running.
ui-reject-not-player-controlled = That character is not yours.
ui-reject-not-awaiting-command = That character is not ready yet.
ui-reject-unknown-action = That character does not know this action.
ui-reject-silenced = Silenced: no magic.
ui-reject-not-enough-mp = Not enough MP.
ui-reject-out-of-item = None left.
ui-reject-invalid-target = Invalid target.
ui-error-bad-message = The server did not understand the request.
ui-error-sandbox-disabled = The sandbox is not available on this server.
ui-error-unknown-scenario = Scenario not found.
ui-error-invalid-speed = Speed not allowed.

## Debug panel

ui-debug-title = Debug
ui-debug-tick = Tick { $tick }
ui-debug-seed = Seed
ui-debug-pause = Pause
ui-debug-resume = Resume
ui-debug-step = Step
ui-debug-speed = Speed
ui-debug-time-mode = Time
ui-time-active = Active
ui-time-wait = Wait
ui-debug-auto = Auto-battle
ui-debug-restart = Restart
ui-debug-fill = ATB
ui-debug-knock-out = KO
ui-debug-apply-status = Status
ui-debug-status = Status to apply
ui-debug-log = Event log

## Event log

ui-log-action = { $actor } uses { $action } on { $targets }
ui-log-damage = { $target } takes { $amount }{ $tags }
ui-log-tag-critical = critical
ui-log-tag-distracted = distracted
ui-log-tag-weak = weak
ui-log-tag-resist = resisted
ui-log-tag-immune = immune
ui-log-heal = { $target } recovers { $amount } HP
ui-log-mp = { $target } recovers { $amount } MP
ui-log-revive = { $target } is revived with { $hp } HP
ui-log-poison = { $target } loses { $amount } HP to poison
ui-log-regen = { $target } regenerates { $amount } HP
ui-log-miss = Miss on { $target }
ui-log-status-gain = { $target } gains { $status }
ui-log-status-lose = { $target } loses { $status }
ui-log-resist = { $target } resists { $status }
ui-log-immune = { $target } is immune to { $status }
ui-log-defend = { $actor } defends
ui-log-limit = { $actor }: Limit ready
ui-log-knocked-out = { $target } is knocked out
ui-log-failed = { $actor } fails { $action }
ui-log-cancelled = { $actor } cancels { $action }
