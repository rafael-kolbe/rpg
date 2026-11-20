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
ui-popup-cover = Cover
ui-popup-stolen = Stolen!
ui-popup-fled = Fled!
ui-popup-recovered = Recovered!
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
ui-status-cover = Cover

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
ui-log-cover = { $protector } covers { $target }
ui-log-stole = { $actor } steals { $item } from { $target }
ui-log-fled = { $actor } runs off with the loot
ui-log-recovered = { $actor } drops { $item }
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
ui-error-not-logged-in = Log in to play.
ui-error-mode-locked = The story has not opened this mode yet.
ui-error-no-game = No game is open.
ui-error-unavailable = The server could not load or save the game. Try again later.
ui-error-opened-elsewhere = The game was opened in another tab or device. Reload to play here.
ui-error-too-many-messages = Too many messages: slow down.

## Game

ui-play = Play
ui-mode-campaign = Campaign
ui-mode-multiplayer = Multiplayer
ui-mode-new = New game
ui-mode-continue = Continue
ui-mode-locked = Locked
ui-mode-locked-hint = The Multiplayer opens later in the Campaign's story.
ui-kalds = { $amount } { $amount ->
        [one] Kald
       *[other] Kalds
    }
ui-continue = Continue
ui-close = Close
ui-world-map = World map
ui-time-mode = Battle time
ui-map-keys = Arrows walk · Enter/Z use · Esc party · M world map
ui-node-exit = Go to { $place }
ui-node-battle = Fight
ui-node-boss = Face the boss
ui-node-ambush = Ambush
ui-node-chest = Open the chest
ui-node-npc = Talk to { $name }
ui-node-shop = Shop
ui-node-temple = Rest and save here
ui-shop-title = Shop
ui-shop-buy = Buy
ui-shop-sell = Sell
ui-shop-nothing = Nothing to sell.
ui-shop-line = { $price } · owned { $owned }
ui-shop-keys = ↑↓ choose · Enter/Z buy or sell one · ←/→ buy or sell · Esc leave
ui-shop-leave = Leave
ui-victory-title = Victory!
ui-victory-xp = { $xp } experience
ui-victory-kalds = { $amount } { $amount ->
        [one] Kald
       *[other] Kalds
    }
ui-found-item = { $name } ×{ $count }
ui-level-up = { $name } reached level { $level }!
ui-learned = Learned { $action }
ui-defeat-title = Defeated…
ui-defeat-text = The party wakes up at the last temple, healed.
ui-rested-title = Rested
ui-rested-text = The party is healed. If you fall, you will return here.
ui-found-title = Found
ui-party-title = Party
ui-party-level = Level { $level }
ui-party-xp = { $xp } / { $next } experience
ui-party-weapon = Weapon
ui-party-armor = Armor
ui-party-none = (none)
ui-party-no-actions = No technics or spells yet
ui-party-use = { $item } ({ $count })
ui-stat-attack = Attack
ui-stat-defense = Defense
ui-stat-magic-attack = Magic
ui-stat-magic-defense = Magic Def.
ui-stat-dexterity = Dexterity
ui-stat-luck = Luck
ui-game-error-busy = Not now.
ui-game-error-no_such_node = You can't go there.
ui-game-error-no_trail = There's no path that way.
ui-game-error-trail_closed = That way is blocked for now.
ui-game-error-barrier = The Campaign has not opened this way yet.
ui-game-error-nothing_here = Nothing to do here.
ui-game-error-no_such_choice = That answer doesn't exist.
ui-game-error-not_for_sale = The shop doesn't trade that.
ui-game-error-not_enough_kalds = Not enough Kalds.
ui-game-error-not_owned = You don't have that.
ui-game-error-cannot_equip = That hero can't use it.
ui-game-error-not_usable_here = It won't do anything now.
ui-game-error-no_such_hero = No such hero.
ui-game-error-bag_full = You can't carry more of that.

# Missions, ranks, Game Over
ui-node-inn = Rest ({ $price } { $price ->
        [one] Kald
       *[other] Kalds
    })
ui-node-board = See the missions
ui-node-sealed = Closed for now.
ui-board-title = Mission board
ui-board-empty = No missions posted.
ui-board-accept = Take
ui-board-locked = Rank too low
ui-board-taken = Under way
ui-board-done = Done
ui-quest-done = Mission complete: { $name }
ui-game-over-title = Game Over
ui-game-over-text = The party has fallen. Continue goes back to the last save.
ui-game-error-not_on_board = That mission is not on this board.
ui-game-error-rank_too_low = Your Guild rank is too low for that.
ui-game-error-quest_taken = You already took that mission.
ui-game-error-sealed = Closed for now.
ui-rank-bronze = Bronze
ui-rank-silver = Silver
ui-rank-gold = Gold
ui-rank-platinum = Platinum
ui-rank-diamond = Diamond
ui-rank-orichalcum = Orichalcum
ui-rank-dracite = Dracite
