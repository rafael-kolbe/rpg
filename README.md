# RPG

Browser RPG with ATB battles. Rust server, Postgres, Vite + React client.

- Game design: [`PLAN.md`](PLAN.md) (Portuguese)
- Project map and working rules: [`CLAUDE.md`](CLAUDE.md)
- Decisions: [`docs/decisions/`](docs/decisions/)

## Requirements

- Rust via [rustup](https://rustup.rs) (the version is pinned in `rust-toolchain.toml`)
- Node.js 22+
- Docker with Compose

## Quick start

```bash
make help           # every command

# Development (hot reload)
cd web && npm install && cd ..
make dev-db         # Postgres on 127.0.0.1:5433
make dev-server     # server on http://127.0.0.1:3000
make dev-web        # client on http://127.0.0.1:5173

# Production-like (built image)
cp .env.example .env   # set POSTGRES_PASSWORD
make up                # http://127.0.0.1:8083

make check          # everything that must pass before a change is done
```
