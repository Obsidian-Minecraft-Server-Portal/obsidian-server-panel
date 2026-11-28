# syntax=docker/dockerfile:1

ARG NODE_VERSION=22
ARG RUST_VERSION=1.97
ARG PNPM_VERSION=12.8.1

FROM node:${NODE_VERSION}-bookworm-slim AS frontend
ARG PNPM_VERSION
WORKDIR /src
RUN corepack enable && corepack prepare pnpm@${PNPM_VERSION} --activate
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml .npmrc ./
RUN pnpm install --frozen-lockfile
COPY index.html tsconfig.json tsconfig.node.json vite.config.ts ./
COPY src ./src
COPY public ./public
RUN pnpm run build:frontend

FROM rust:${RUST_VERSION}-bookworm AS build
ARG VERSION=""
WORKDIR /src
COPY . .
COPY --from=frontend /src/target/wwwroot ./target/wwwroot
RUN if [ -n "$VERSION" ]; then sh packaging/stamp-version.sh "$VERSION"; fi
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/cargo-target,sharing=locked \
    --mount=type=secret,id=CURSEFORGE_API_KEY,env=CURSEFORGE_API_KEY,required=false \
    CARGO_TARGET_DIR=/cargo-target cargo build --release --locked \
        --no-default-features --features docker --bin obsidian \
    && cp /cargo-target/release/obsidian /usr/local/bin/obsidian

FROM debian:bookworm-slim AS runtime
LABEL org.opencontainers.image.source="https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel" \
      org.opencontainers.image.description="Obsidian Minecraft server panel"
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tini \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --user-group --home-dir /data --shell /usr/sbin/nologin obsidian \
    && mkdir /data \
    && chown obsidian:obsidian /data
COPY --from=build /usr/local/bin/obsidian /usr/local/bin/obsidian
ENV OBSIDIAN_DATA_DIR=/data
USER obsidian
WORKDIR /data
VOLUME ["/data"]
EXPOSE 1421 2022
ENTRYPOINT ["tini", "--", "obsidian"]
CMD ["serve"]
