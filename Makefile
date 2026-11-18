# RPG — entry points for running, developing and checking the project.
#
# Same convention as the other services on this machine:
#   make up    foreground, logs on screen, Ctrl+C stops
#   make up-d  same, in the background

SHELL   := /bin/bash
COMPOSE := docker compose -p rpg -f docker-compose.yml
DEV     := docker compose -p rpg-dev -f docker-compose.dev.yml
NETWORK := proxy_net
DEV_DATABASE_URL := postgres://rpg:rpg@127.0.0.1:5433/rpg

# rustup installs cargo in ~/.cargo/bin; make sure recipes find it even from a
# shell that has not loaded ~/.cargo/env.
export PATH := $(HOME)/.cargo/bin:$(PATH)

# The backup container writes ./backups as the host user (see docker-compose.yml).
export HOST_UID := $(shell id -u)
export HOST_GID := $(shell id -g)

.PHONY: help network require-env up up-d down logs ps build \
        dev-db dev-db-down dev-server dev-web \
        check check-rust check-content check-web check-protocol check-skipped-tests check-utils-docs \
        fmt docs protocol-types sim e2e

help:
	@echo "Run (production-like, http://127.0.0.1:8083):"
	@echo "  make up            build and run app + db + backup in the foreground"
	@echo "  make up-d          same, in the background"
	@echo "  make down          stop everything (data is kept)"
	@echo "  make logs          follow logs (LOGS=app|db|backup)"
	@echo "  make build         rebuild the image"
	@echo ""
	@echo "Develop (hot reload):"
	@echo "  make dev-db        start the development database (127.0.0.1:5433)"
	@echo "  make dev-server    run the server with cargo (http://127.0.0.1:3000)"
	@echo "  make dev-web       run the client with Vite (http://127.0.0.1:5173)"
	@echo "  make dev-db-down   stop the development database"
	@echo ""
	@echo "Quality:"
	@echo "  make check         everything that must pass before a change is done"
	@echo "  make fmt           format Rust and web code"
	@echo "  make docs          build and open the code documentation"
	@echo "  make e2e           build the client and run the browser tests (Playwright)"
	@echo "  make protocol-types  regenerate TypeScript types from crates/protocol"
	@echo ""
	@echo "Battle simulator (AI vs AI on a sandbox scenario):"
	@echo "  make sim           one battle, full log (SEED=42 SCENARIO=sandbox/battle-basic.json)"
	@echo "  make sim RUNS=1000 summary over many seeds"

# --- Run ----------------------------------------------------------------------

# External network shared with the my-server proxy; created if missing.
network:
	@docker network inspect $(NETWORK) >/dev/null 2>&1 || docker network create $(NETWORK) >/dev/null

require-env:
	@test -f .env || (echo "Missing .env: cp .env.example .env and set POSTGRES_PASSWORD" && exit 1)

up: network require-env
	$(COMPOSE) up --build

up-d: network require-env
	$(COMPOSE) up --build -d

down:
	$(COMPOSE) down

logs:
	$(COMPOSE) logs -f --tail=100 $(LOGS)

ps:
	$(COMPOSE) ps

build: require-env
	$(COMPOSE) build

# --- Develop ------------------------------------------------------------------

dev-db:
	$(DEV) up -d --wait

dev-db-down:
	$(DEV) down

dev-server:
	DATABASE_URL=$(DEV_DATABASE_URL) WEB_DIR=web/dist cargo run --features sandbox --bin rpg-server

dev-web:
	cd web && npm run dev

# --- Quality ------------------------------------------------------------------

check: check-rust check-content check-protocol check-web check-skipped-tests check-utils-docs
	@echo ""
	@echo "make check: all green."

check-rust:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --locked -- -D warnings
	cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
	cargo test --workspace --all-features --locked --quiet
	cargo machete
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --document-private-items --locked --quiet

check-content:
	cargo run --quiet --locked -p content --bin content-check

check-protocol:
	./scripts/check-protocol-types.sh

check-web:
	cd web && npm run --silent check

check-skipped-tests:
	./scripts/check-no-skipped-tests.sh

check-utils-docs:
	./scripts/check-utils-docs.sh

# Browser tests: Playwright starts the server with the sandbox feature itself.
e2e:
	cd web && npm run --silent build && npx playwright test

fmt:
	cargo fmt --all
	cd web && npm run --silent format

docs:
	cargo doc --workspace --no-deps --document-private-items --open

protocol-types:
	cargo test --quiet -p protocol

SEED     ?= 42
RUNS     ?= 1
SCENARIO ?= sandbox/battle-basic.json

sim:
	cargo run --quiet --release -p sim -- --scenario $(SCENARIO) --seed $(SEED) --runs $(RUNS)
