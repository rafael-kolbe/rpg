# syntax=docker/dockerfile:1

# --- Server: release build of the Rust binary --------------------------------
FROM rust:1.100-bookworm AS server-build
# "true" builds the sandbox (test scenarios + debug tools). Local testing only:
# a published image must be built with the default.
ARG SANDBOX=false
WORKDIR /src
COPY rust-toolchain.toml Cargo.toml Cargo.lock clippy.toml rustfmt.toml ./
COPY crates crates
# Queries are checked against .sqlx/ (no database at build time); the
# migrations are embedded in the binary.
COPY .sqlx .sqlx
COPY migrations migrations
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked --bin rpg-server \
      $( [ "$SANDBOX" = "true" ] && echo "--features sandbox" ) \
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
# Read by sandbox builds only; tiny, so always shipped.
COPY content /app/content
COPY sandbox /app/sandbox
ENV PORT=3000 \
    WEB_DIR=/app/web \
    CONTENT_DIR=/app/content \
    SANDBOX_DIR=/app/sandbox \
    LOG_FORMAT=json
EXPOSE 3000
ENTRYPOINT ["/app/rpg-server"]
