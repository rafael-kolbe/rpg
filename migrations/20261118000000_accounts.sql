-- Player accounts (PLAN.md §5 "Contas", ADR 0007): username and password,
-- no e-mail. Secrets are stored only as hashes.
CREATE TABLE accounts (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- As typed at sign-up (shown back to the player).
    username TEXT NOT NULL,
    -- Argon2id, PHC string format.
    password_hash TEXT NOT NULL,
    -- SHA-256 of the normalized recovery key.
    recovery_key_hash BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Usernames are unique regardless of case.
CREATE UNIQUE INDEX accounts_username_unique ON accounts (lower(username));

-- Logged-in browsers. The cookie holds the token; only its SHA-256 is here.
CREATE TABLE sessions (
    token_hash BYTEA PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX sessions_account_id ON sessions (account_id);
