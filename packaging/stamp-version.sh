#!/bin/sh
# Usage: stamp-version.sh <tag>  (e.g. v1.2.3); rewrites the obsidian package version in Cargo.toml and Cargo.lock.
set -eu
VERSION="${1#v}"
export VERSION
perl -0pi -e 's/^version = "[^"]*"/version = "$ENV{VERSION}"/m' Cargo.toml
perl -0pi -e 's/(name = "obsidian"\r?\nversion = )"[^"]*"/$1"$ENV{VERSION}"/' Cargo.lock
grep -q "^version = \"$VERSION\"" Cargo.toml
