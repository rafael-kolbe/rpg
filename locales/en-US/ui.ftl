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
ui-command-magic = Magic
ui-command-technic = Technic
ui-command-item = Item
ui-command-back = Back
ui-command-choose-target = Choose a target
ui-command-waiting = Waiting for a turn…
ui-battle-keys = ↑↓ choose · Enter/Z confirm (hold: repeat last) · Esc/X back · ← defend · Tab next hero · M memory
ui-battle-sound = Sound: { $state ->
    [on] on
   *[off] off
}
ui-battle-memory = Action memory: { $state ->
    [on] on
   *[off] off
}
ui-intent-defend = Defend
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
ui-time-turn = Turn
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
ui-log-knocked-out = { $target } is knocked out
ui-log-failed = { $actor } fails { $action }
ui-log-cancelled = { $actor } cancels { $action }

## Account

ui-account-sign-up = Create account
ui-account-log-in = Log in
ui-account-log-out = Log out
ui-account-recover = Change password with the recovery key
ui-account-forgot = Forgot your password?
ui-account-have-account = Already have an account? Log in
ui-account-logged-in = Logged in as { $username }
ui-account-username = Username
ui-account-username-rule = 8 to 20 letters and numbers, no spaces or symbols.
ui-account-password-current = Password
ui-account-password-new = New password
ui-account-password-confirm = Repeat the password
ui-account-password-rule = 8 to 64 characters.
ui-account-recovery-key = Recovery key
ui-account-error-mismatch = The two passwords are different.
ui-account-error-invalid_username = The username needs 8 to 20 letters and numbers, nothing else.
ui-account-error-invalid_password = The password needs 8 to 64 characters.
ui-account-error-username_taken = That username is taken.
ui-account-error-wrong_credentials = Wrong username or password.
ui-account-error-wrong_recovery_key = Wrong username or recovery key.
ui-account-error-too_many_attempts = Too many failed attempts. Wait 15 minutes and try again.
ui-account-error-not_logged_in = You are not logged in.
ui-account-error-unavailable = The server is unavailable. Try again later.
ui-recovery-key-title = Your recovery key
ui-recovery-key-explain = This key is the only way to change your password if you forget it. It is shown only now.
ui-recovery-key-copy = Copy
ui-recovery-key-copied = Copied
ui-recovery-key-warning = Keep it somewhere safe (a password manager, a note on paper). Without it, a forgotten password means a lost account. Never share it.
ui-recovery-key-saved = I saved my recovery key
ui-recovery-key-continue = Continue
