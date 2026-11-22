set windows-shell := ["powershell.exe", "-NoProfile", "-NoLogo", "-Command"]
set shell := ["bash", "-c"]

@default:
    @just --list

install:
    @pnpm i

dev: install
    @cargo run

check:
    @cargo check --workspace

clippy:
    @cargo clippy --workspace --all-targets

test:
    @cargo test --workspace

build-frontend: install
    @pnpm tsc
    @pnpm vite build

build: build-frontend
    @cargo build --workspace --release
