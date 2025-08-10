echo Installing prerequisites...
sudo apt install jq p7zip-full curl -y > /dev/null

echo Downloading obsidian artifacts
download_url="$(curl -s https://api.github.com/repos/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/releases | jq -r '.[0].assets[] | select(.name=="obsidian-linux-x64.zip") | .browser_download_url')"
curl -Lso obsidian.zip $download_url > /dev/null

echo Extracting to "$(pwd)/obsidian/"
7z e -oobsidian -y obsidian.zip > /dev/null
rm obsidian.zip > /dev/null
cd "$(pwd)/obsidian/"

read -p "What should the WebUI Port be (default: 80): " port; : "${port:=80}"
read -p "Enable UPNP Portforward WebUI? (y/N): " yn

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
sudo echo "$service_text" > /etc/systemd/system/obsidian.service
sudo systemctl daemon-reload
sudo systemctl start obsidian