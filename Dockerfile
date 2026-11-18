# syntax=docker/dockerfile:1

# --- Server: release build of the Rust binary --------------------------------
FROM rust:1.100-bookworm AS server-build
WORKDIR /src
COPY rust-toolchain.toml Cargo.toml Cargo.lock clippy.toml rustfmt.toml ./
COPY crates crates
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked --bin rpg-server \
 && cp target/release/rpg-server /rpg-server

# --- Web: production build of the client --------------------------------------
FROM node:22-bookworm-slim AS web-build
WORKDIR /src/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY locales /src/locales
COPY web ./
RUN npm run build

# --- Runtime: distroless, no shell, non-root ----------------------------------
FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=server-build /rpg-server /app/rpg-server
COPY --from=web-build /src/web/dist /app/web
ENV PORT=3000 \
    WEB_DIR=/app/web \
    LOG_FORMAT=json
EXPOSE 3000
ENTRYPOINT ["/app/rpg-server"]
