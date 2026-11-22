set windows-shell := ["powershell.exe", "-NoProfile", "-NoLogo", "-Command"]
set shell := ["bash", "-c"]

@defualt:
    @just --list

install:
    @pnpm i

build: install
    @pnpm tsc
    @pnpm vite build
    @cargo build --workspace --release

dev: install
    @cargo run --profile dev