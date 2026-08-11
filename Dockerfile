FROM rust:1.92-alpine AS builder
WORKDIR /build

# Database backend: sqlite, mysql (default), or postgres
ARG DB_FEATURE=mysql

RUN apk add --no-cache nodejs npm musl-dev openssl-dev openssl-libs-static pkgconfig perl && \
    npm i -g pnpm

COPY package.json pnpm-lock.yaml ./
RUN pnpm i --no-frozen-lockfile

COPY . .

RUN pnpm run build:frontend
RUN cargo build --release --features "${DB_FEATURE}"

# ── Runtime ──────────────────────────────────────────────────
FROM alpine:3.22

RUN apk add --no-cache ca-certificates screen && \
    mkdir -p /app/meta/servers /app/meta/java /app/meta/backups /app/meta/temp

COPY --from=builder /build/target/release/obsidian_server_panel /app/obsidian_server_panel

WORKDIR /app
EXPOSE 80

CMD ["/app/obsidian_server_panel", "--port", "80"]
