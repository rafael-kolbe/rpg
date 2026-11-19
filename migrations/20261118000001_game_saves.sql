-- Campaign and Multiplayer saves (PLAN.md §4 "Regras do arco"): the story
-- progress is shared by the account's two modes; the rest is per mode.
-- Both are JSON documents owned by the `world` crate (`Progress`, `Save`).
CREATE TABLE progress (
    account_id BIGINT PRIMARY KEY REFERENCES accounts (id) ON DELETE CASCADE,
    data JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE saves (
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    -- `campaign` or `multiplayer`.
    mode TEXT NOT NULL CHECK (mode IN ('campaign', 'multiplayer')),
    data JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, mode)
);
