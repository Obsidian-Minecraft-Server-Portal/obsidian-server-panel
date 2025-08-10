#!/usr/bin/env bash
# Exit immediately on errors (-e), treat unset variables as errors (-u), and
# fail a pipeline if any command errors (-o pipefail).
set -euo pipefail

echo "Checking for available download/extract tools..."

# Choose a downloader: prefer curl, fallback to wget. If neither exists, abort.
downloader=""
if command -v curl >/dev/null 2>&1; then
  downloader="curl -fsSL" # -f: fail on HTTP errors, -sS: silent with errors, -L: follow redirects
elif command -v wget >/dev/null 2>&1; then
  downloader="wget -qO-"  # -q: quiet, -O-: write to stdout
else
  echo "Error: Need either curl or wget to download files, but neither is installed."
  echo "Please install curl or wget and re-run."
  exit 1
fi

# Choose a .zip extractor: prefer unzip, fallback to bsdtar or 7z. If none exists, abort.
extractor=""
if command -v unzip >/dev/null 2>&1; then
  extractor="unzip -o"         # -o: overwrite existing files without prompting
elif command -v bsdtar >/dev/null 2>&1; then
  extractor="bsdtar -xf"       # -x: extract, -f: file
elif command -v 7z >/dev/null 2>&1; then
  extractor="7z x -y"          # x: extract with full paths, -y: assume Yes on all queries
else
  echo "Error: Need an extractor for .zip files (unzip, bsdtar, or 7z), but none is installed."
  echo "Please install one of them and re-run."
  exit 1
fi

echo "Downloading Obsidian artifacts..."

# GitHub API endpoint for repository releases (returns JSON)
releases_url="https://api.github.com/repos/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/releases"

# Fetch the releases JSON; if it fails, abort.
json="$($downloader "$releases_url")" || { echo "Failed to fetch releases info."; exit 1; }

# Parse the JSON using awk/sed to find the browser_download_url that matches obsidian-linux-x64.zip
# Note: This avoids requiring jq by doing a simple line-based search.
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

# Ensure we found a matching asset URL.
if [ -z "${download_url:-}" ]; then
  echo "Error: Could not find download URL for obsidian-linux-x64.zip in the latest release."
  exit 1
fi

echo "Fetching: $download_url"

# Download the ZIP to a local file using the previously selected tool.
if [[ "$downloader" == curl* ]]; then
  curl -fLso obsidian.zip "$download_url" # -f: fail on server errors, -L: follow redirects, -s: silent, -o: output file
else
  wget -qO obsidian.zip "$download_url"   # -q: quiet, -O: output file
fi

echo "Extracting to $(pwd)/obsidian/"

# Ensure target directory exists.
mkdir -p obsidian

# Extract based on the chosen extractor. Suppress noisy output where possible.
if [[ "$extractor" == "unzip -o" ]]; then
  unzip -o obsidian.zip -d obsidian >/dev/null
elif [[ "$extractor" == "bsdtar -xf" ]]; then
  bsdtar -xf obsidian.zip -C obsidian
else
  7z x -y -oobsidian obsidian.zip >/dev/null
fi

# Remove the ZIP after successful extraction to save space.
rm -f obsidian.zip

# Work inside the extracted directory for the remainder of setup.
cd ./obsidian/

# Prompt for web UI port with default 80 if user presses Enter.
read -rp "What should the WebUI Port be (default: 80): " port; : "${port:=80}"

# Ask if UPnP port forwarding should be enabled for the web panel.
read -N 1 -rp "Enable UPNP port forwarding WebUI? (y/N): " yn; echo
forward_flag=""
if [[ "$yn" =~ ^[Yy]$ ]]; then # If yn is 'y' or 'Y'
  forward_flag="--forward-webpanel "
fi

# Create a systemd service unit content pointing to the extracted binary in the current directory.
# Uses root user/group and restarts automatically on failure.
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

# Write the systemd unit, set correct permissions, reload systemd, and start the service.
echo "$service_text" | sudo tee /etc/systemd/system/obsidian.service >/dev/null
sudo chmod 644 /etc/systemd/system/obsidian.service
sudo systemctl daemon-reload
sudo systemctl start obsidian

echo "Done. Service 'obsidian' started."