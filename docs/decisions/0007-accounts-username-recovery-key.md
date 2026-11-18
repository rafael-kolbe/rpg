# 0007 — Accounts: username, password and a recovery key, no e-mail yet

- **Status:** accepted
- **Date:** 2026-11-18

## Context

The Story Mode needs accounts (the save lives on the server). The options
were our own login, e-mail magic links or social login (Google/Discord).
E-mail confirmation and recovery by e-mail need an SMTP provider and mean
storing personal data; neither is needed to play.

## Decision

- Our own accounts: **username and password**, both at least 8 characters
  (username up to 20, password up to 64); usernames only take ASCII letters
  and digits and are unique, case-insensitive.
- Passwords are stored only as Argon2id hashes.
- At sign-up the server generates a **recovery key**, shown once with a copy
  button and a warning to keep it safe; only its hash is stored. The key is
  the only way to change a forgotten password; using it issues a new one.
- Sessions use an `httpOnly`, `SameSite` cookie that also authenticates the
  WebSocket. Login attempts are rate-limited by username and by IP.
- No e-mail and no social login for now.

## Consequences

- No personal data stored; no mail infrastructure to run.
- A player who loses both the password and the recovery key loses the
  account. The sign-up screen has to make that clear.
- Adding e-mail later (confirmation, recovery) is additive: a nullable column
  and a new flow, nothing to migrate away from.
