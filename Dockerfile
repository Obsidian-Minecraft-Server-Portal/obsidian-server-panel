FROM rust:1.92-alpine AS builder
WORKDIR /build

# Database backend: sqlite, mysql (default), or postgres
ARG DB_FEATURE=mysql

RUN apk add --no-cache nodejs npm musl-dev openssl-dev openssl-libs-static pkgconfig perl && \
    npm i -g pnpm@10

COPY package.json pnpm-lock.yaml ./
RUN pnpm i --no-frozen-lockfile

COPY . .

RUN node node_modules/typescript/bin/tsc && node node_modules/vite/bin/vite.js build
RUN cargo build --release --no-default-features --features "${DB_FEATURE}"

# ── Runtime ──────────────────────────────────────────────────
FROM alpine:3.22

RUN apk add --no-cache ca-certificates screen && \
    mkdir -p /app/meta/servers /app/meta/java /app/meta/backups /app/meta/temp

COPY --from=builder /build/target/release/obsidian_server_panel /app/obsidian_server_panel

WORKDIR /app
EXPOSE 80

CMD ["/app/obsidian_server_panel", "--port", "80"]
