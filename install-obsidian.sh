#!/usr/bin/env bash
set -euo pipefail

echo "Checking for available download/extract tools..."

# Pick a downloader: curl or wget (no auto-install)
downloader=""
if command -v curl >/dev/null 2>&1; then
  downloader="curl -fsSL"
elif command -v wget >/dev/null 2>&1; then
  downloader="wget -qO-"
else
  echo "Error: Need either curl or wget to download files, but neither is installed."
  echo "Please install curl or wget and re-run."
  exit 1
fi

# Pick an extractor: unzip, bsdtar, or 7z (whichever exists)
extractor=""
if command -v unzip >/dev/null 2>&1; then
  extractor="unzip -o"
elif command -v bsdtar >/dev/null 2>&1; then
  extractor="bsdtar -xf"
elif command -v 7z >/dev/null 2>&1; then
  extractor="7z x -y"
else
  echo "Error: Need an extractor for .zip files (unzip, bsdtar, or 7z), but none is installed."
  echo "Please install one of them and re-run."
  exit 1
fi

echo "Downloading Obsidian artifacts..."
# Get latest releases JSON and extract the browser_download_url for obsidian-linux-x64.zip without jq.
releases_url="https://api.github.com/repos/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/releases"
json="$($downloader "$releases_url")" || { echo "Failed to fetch releases info."; exit 1; }

# Grep/sed to find the asset with name obsidian-linux-x64.zip and get its browser_download_url
download_url="$(
  printf "%s" "$json" \
  | tr -d '\r' \
  | awk '
      /"name":/ { name=$0 }
      /"browser_download_url":/ {
        url=$0
        if (name ~ /obsidian-linux-x64\.zip"/) {
          print url
          exit
        }
      }
    ' \
  | sed -n 's/.*"browser_download_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p'
)"

if [ -z "${download_url:-}" ]; then
  echo "Error: Could not find download URL for obsidian-linux-x64.zip in the latest release."
  exit 1
fi

# Download the asset to a file
echo "Fetching: $download_url"
if [[ "$downloader" == curl* ]]; then
  curl -fLso obsidian.zip "$download_url"
else
  wget -qO obsidian.zip "$download_url"
fi

echo "Extracting to $(pwd)/obsidian/"
mkdir -p obsidian
if [[ "$extractor" == "unzip -o" ]]; then
  unzip -o obsidian.zip -d obsidian >/dev/null
elif [[ "$extractor" == "bsdtar -xf" ]]; then
  bsdtar -xf obsidian.zip -C obsidian
else
  7z x -y -oobsidian obsidian.zip >/dev/null
fi
rm -f obsidian.zip

cd ./obsidian/

read -rp "What should the WebUI Port be (default: 80): " port; : "${port:=80}"
read -N 1 -rp "Enable UPNP Portforward WebUI? (y/N): " yn; echo

forward_flag=""
if [[ "$yn" =~ ^[Yy]$ ]]; then
  forward_flag="--forward-webpanel "
fi

service_text="
[Unit]
Description=Obsidian Minecraft Server Panel
Wants=network-online.target
After=network-online.target

[Service]
Type=simple
User=root
Group=root
ExecStart=$(pwd)/obsidian_server_panel ${forward_flag}-p $port
WorkingDirectory=$(pwd)
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
"

echo "Creating systemd service file..."
# Use tee so redirection happens with sudo privileges
echo "$service_text" | sudo tee /etc/systemd/system/obsidian.service >/dev/null
sudo chmod 644 /etc/systemd/system/obsidian.service
sudo systemctl daemon-reload
sudo systemctl start obsidian

echo "Done. Service 'obsidian' started."