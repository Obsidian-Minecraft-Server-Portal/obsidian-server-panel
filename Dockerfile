FROM rust:1.92-alpine AS builder
WORKDIR /build

# Database backend: sqlite (default), mysql, or postgres
ARG DB_FEATURE=sqlite

COPY . .

RUN apk update && \
    apk add --no-cache nodejs npm musl-dev openssl-dev openssl-libs-static pkgconfig perl

RUN npm i vite pnpm -g
RUN pnpm i --no-frozen-lockfile --force
RUN pnpm run "build:frontend"
RUN cargo build --release --features "${DB_FEATURE}"

# ── Runtime ──────────────────────────────────────────────────
FROM alpine:latest

RUN apk add --no-cache ca-certificates && \
    mkdir -p /app/meta/servers /app/meta/java /app/meta/backups /app/meta/temp

COPY --from=builder /build/target/release/obsidian_server_panel /app/obsidian_server_panel
RUN chmod +x /app/obsidian_server_panel

WORKDIR /app
EXPOSE 80

CMD ["/app/obsidian_server_panel", "--port", "80"]
