#!/bin/sh
# Installs the latest Obsidian release binary on Linux or macOS.
#   curl -fsSL https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/releases/latest/download/install.sh | sh
# Environment overrides:
#   OBSIDIAN_VERSION   release tag to install (default: latest)
#   OBSIDIAN_PREFIX    directory for the obsidian command (default: /usr/local/bin)
#   OBSIDIAN_DESKTOP=1 Linux: install the build with the system tray icon (needs GTK and an appindicator library)
#   OBSIDIAN_SERVICE=1 Linux: run the panel as the "obsidian" systemd service from /opt/obsidian
#   OBSIDIAN_DATA_DIR  service data directory (default: /var/lib/obsidian)
set -eu

REPO="Obsidian-Minecraft-Server-Portal/obsidian-server-panel"
VERSION="${OBSIDIAN_VERSION:-latest}"
PREFIX="${OBSIDIAN_PREFIX:-/usr/local/bin}"
DATA_DIR="${OBSIDIAN_DATA_DIR:-/var/lib/obsidian}"
SERVICE_HOME=/opt/obsidian
SERVICE_USER=obsidian

die() {
	echo "error: $*" >&2
	exit 1
}

if [ "$VERSION" = latest ]; then
	BASE="https://github.com/$REPO/releases/latest/download"
else
	BASE="https://github.com/$REPO/releases/download/$VERSION"
fi

case "$(uname -m)" in
	x86_64 | amd64) ARCH=x86_64 ;;
	aarch64 | arm64) ARCH=aarch64 ;;
	*) die "unsupported architecture: $(uname -m)" ;;
esac

FLAVOR=""
case "$(uname -s)" in
	Linux)
		TARGET="$ARCH-unknown-linux-gnu"
		if [ "${OBSIDIAN_DESKTOP:-0}" = 1 ]; then FLAVOR="desktop-"; fi
		;;
	Darwin) TARGET="$ARCH-apple-darwin" ;;
	*) die "unsupported operating system: $(uname -s) (use the Windows installer)" ;;
esac

SERVICE="${OBSIDIAN_SERVICE:-0}"
if [ "$SERVICE" = 1 ]; then
	[ "$(uname -s)" = Linux ] || die "OBSIDIAN_SERVICE is Linux only; on macOS run: sudo $PREFIX/obsidian service install"
	command -v systemctl >/dev/null 2>&1 || die "systemctl not found"
fi

command -v curl >/dev/null 2>&1 || die "curl is required"
command -v tar >/dev/null 2>&1 || die "tar is required"
sha256() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$1" | cut -d' ' -f1
	else
		shasum -a 256 "$1" | cut -d' ' -f1
	fi
}

SUDO=""
if [ "$(id -u)" -ne 0 ]; then
	command -v sudo >/dev/null 2>&1 || die "run as root or install sudo"
	SUDO=sudo
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

ARCHIVE="obsidian-$FLAVOR$TARGET.tar.gz"
curl -fsSL -o "$TMP/$ARCHIVE" "$BASE/$ARCHIVE"
curl -fsSL -o "$TMP/SHA256SUMS" "$BASE/SHA256SUMS"
EXPECTED=$(awk -v file="$ARCHIVE" '$2 == file { print $1 }' "$TMP/SHA256SUMS")
[ -n "$EXPECTED" ] || die "no checksum published for $ARCHIVE"
[ "$(sha256 "$TMP/$ARCHIVE")" = "$EXPECTED" ] || die "checksum mismatch for $ARCHIVE"
tar -xzf "$TMP/$ARCHIVE" -C "$TMP"

$SUDO mkdir -p "$PREFIX"
if [ "$SERVICE" != 1 ]; then
	$SUDO install -m 0755 "$TMP/obsidian" "$PREFIX/obsidian"
	echo "installed $("$PREFIX/obsidian" --version) to $PREFIX/obsidian"
	exit 0
fi

# The service account owns its install directory so the panel can update itself in place.
id "$SERVICE_USER" >/dev/null 2>&1 \
	|| $SUDO useradd --system --user-group --home-dir "$DATA_DIR" --shell /usr/sbin/nologin "$SERVICE_USER"
$SUDO install -d -m 0755 -o "$SERVICE_USER" -g "$SERVICE_USER" "$SERVICE_HOME" "$DATA_DIR"
$SUDO install -m 0755 -o "$SERVICE_USER" -g "$SERVICE_USER" "$TMP/obsidian" "$SERVICE_HOME/obsidian"
$SUDO ln -sf "$SERVICE_HOME/obsidian" "$PREFIX/obsidian"
echo "installed $("$SERVICE_HOME/obsidian" --version) to $SERVICE_HOME/obsidian"

curl -fsSL -o "$TMP/obsidian.service" "$BASE/obsidian.service"
sed -e "s|/opt/obsidian/obsidian|$SERVICE_HOME/obsidian|g" -e "s|/var/lib/obsidian|$DATA_DIR|g" "$TMP/obsidian.service" >"$TMP/unit"
$SUDO install -m 0644 "$TMP/unit" /etc/systemd/system/obsidian.service
$SUDO systemctl daemon-reload
$SUDO systemctl enable obsidian.service
$SUDO systemctl restart obsidian.service
echo "service running; the panel listens on port 1421 by default"
