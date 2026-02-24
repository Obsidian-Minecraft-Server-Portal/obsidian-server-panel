FROM rust:1.90-alpine AS builder
ARG TMDB_API_KEY
WORKDIR /build
COPY . .
RUN apk update

# Install Dependencies
RUN apk add --no-cache nodejs npm musl-dev
RUN npm i vite pnpm -g

# Install project dependencies (force clean install without prompts)
RUN pnpm i --no-frozen-lockfile --force
# Build the project
# This will build both the frontend and the backend
RUN pnpm run "build:frontend"
RUN cargo build --release
RUN strip target/release/obsidian_server_panel

FROM alpine:latest
ENV SERVER_PORT=3698
RUN mkdir -p /app
COPY --from=builder /build/target/release/obsidian_server_panel /app/obsidian_server_panel
RUN chmod +x /app/obsidian_server_panel
WORKDIR /app
EXPOSE 3698
CMD /app/obsidian_server_panel