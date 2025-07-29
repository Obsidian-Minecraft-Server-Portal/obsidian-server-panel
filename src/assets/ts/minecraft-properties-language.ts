import * as monaco from "monaco-editor";

export const minecraftPropertiesConfig: monaco.languages.LanguageConfiguration = {
    comments: {
        lineComment: "#"
    },
    brackets: [
        ["[", "]"],
        ["(", ")"]
    ],
    autoClosingPairs: [
        {open: "[", close: "]"},
        {open: "(", close: ")"},
        {open: "\"", close: "\""},
        {open: "'", close: "'"}
    ],
    surroundingPairs: [
        {open: "[", close: "]"},
        {open: "(", close: ")"},
        {open: "\"", close: "\""},
        {open: "'", close: "'"}
    ]
};

export const minecraftPropertiesLanguage: monaco.languages.IMonarchLanguage = {
    tokenizer: {
        root: [
            [/#.*$/, "comment"],
            [/^[^=]+(?==)/, "key"],
            [/=/, "operator"],
            [/.*$/, "value"]
        ]
    }
};

const minecraftPropertiesCompletionItems = [
    {
        label: "server-port",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "server-port=25565",
        documentation: "Changes the port the server is hosting (listening) on. Default: 25565"
    },
    {
        label: "max-players",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "max-players=20",
        documentation: "The maximum number of players that can play on the server at the same time. Default: 20"
    },
    {
        label: "gamemode",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "gamemode=survival",
        documentation: "Defines the mode of gameplay. Values: survival, creative, adventure, spectator. Default: survival"
    },
    {
        label: "difficulty",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "difficulty=easy",
        documentation: "Defines the difficulty of the server. Values: peaceful, easy, normal, hard. Default: easy"
    },
    {
        label: "level-name",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "level-name=world",
        documentation: "The \"level-name\" value is used as the world name and its folder name. Default: world"
    },
    {
        label: "level-seed",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "level-seed=",
        documentation: "Sets a world seed for the player's world, as in Singleplayer."
    },
    {
        label: "pvp",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "pvp=true",
        documentation: "Enable PvP on the server. Values: true, false. Default: true"
    },
    {
        label: "online-mode",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "online-mode=true",
        documentation: "Server checks connecting players against Minecraft account database. Values: true, false. Default: true"
    },
    {
        label: "white-list",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "white-list=false",
        documentation: "Enables a whitelist on the server. Values: true, false. Default: false"
    },
    {
        label: "motd",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "motd=A Minecraft Server",
        documentation: "This is the message that is displayed in the server list of the client. Default: A Minecraft Server"
    },
    {
        label: "view-distance",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "view-distance=10",
        documentation: "Sets the amount of world data the server sends the client (2-32). Default: 10"
    },
    {
        label: "spawn-protection",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "spawn-protection=16",
        documentation: "Determines the side length of the square spawn protection area. Default: 16"
    },
    {
        label: "enable-command-block",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "enable-command-block=false",
        documentation: "Enables command blocks. Values: true, false. Default: false"
    },
    {
        label: "allow-nether",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "allow-nether=true",
        documentation: "Allows players to travel to the Nether. Values: true, false. Default: true"
    },
    {
        label: "level-type",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "level-type=default",
        documentation: "Determines the type of map that is generated. Values: default, flat, largeBiomes, amplified, buffet. Default: default"
    },
    {
        label: "generator-settings",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "generator-settings=",
        documentation: "The settings used to customize world generation for flat, buffet, and custom worlds."
    },
    {
        label: "enable-jmx-monitoring",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "enable-jmx-monitoring=false",
        documentation: "Exposes an MBean with the Object name net.minecraft.server:type=Server. Values: true, false. Default: false"
    },
    {
        label: "enable-rcon",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "enable-rcon=false",
        documentation: "Enables remote access to the server console. Values: true, false. Default: false"
    },
    {
        label: "rcon.port",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "rcon.port=25575",
        documentation: "Sets the RCON network port. Default: 25575"
    },
    {
        label: "rcon.password",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "rcon.password=",
        documentation: "Sets the password for RCON. Required if enable-rcon is true."
    },
    {
        label: "query.port",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "query.port=25565",
        documentation: "Sets the query port for the server. Default: 25565"
    },
    {
        label: "enable-query",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "enable-query=false",
        documentation: "Enables GameSpy4 protocol server listener. Values: true, false. Default: false"
    },
    {
        label: "op-permission-level",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "op-permission-level=4",
        documentation: "Sets the default permission level for ops when using /op. Values: 1-4. Default: 4"
    },
    {
        label: "player-idle-timeout",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "player-idle-timeout=0",
        documentation: "Kicks idle players after specified minutes. 0 to disable. Default: 0"
    },
    {
        label: "force-gamemode",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "force-gamemode=false",
        documentation: "Forces players to join in the default game mode. Values: true, false. Default: false"
    },
    {
        label: "hardcore",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "hardcore=false",
        documentation: "If set to true, server difficulty is ignored and set to hard and players are set to spectator mode if they die. Values: true, false. Default: false"
    },
    {
        label: "network-compression-threshold",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "network-compression-threshold=256",
        documentation: "By default it allows packets that are n-1 bytes big to be uncompressed. Default: 256"
    },
    {
        label: "resource-pack",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "resource-pack=",
        documentation: "Optional URI to a resource pack. The player may choose to use it."
    },
    {
        label: "resource-pack-sha1",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "resource-pack-sha1=",
        documentation: "Optional SHA-1 digest of the resource pack, in lowercase hexadecimal."
    },
    {
        label: "spawn-npcs",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "spawn-npcs=true",
        documentation: "Determines if NPCs will be spawned. Values: true, false. Default: true"
    },
    {
        label: "spawn-animals",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "spawn-animals=true",
        documentation: "Determines if animals will be spawned. Values: true, false. Default: true"
    },
    {
        label: "spawn-monsters",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "spawn-monsters=true",
        documentation: "Determines if monsters will be spawned. Values: true, false. Default: true"
    },
    {
        label: "generate-structures",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "generate-structures=true",
        documentation: "Defines whether structures (such as villages) can be generated. Values: true, false. Default: true"
    },
    {
        label: "simulation-distance",
        kind: monaco.languages.CompletionItemKind.Property,
        insertText: "simulation-distance=10",
        documentation: "Sets the maximum distance from players that living entities may be located in order to be updated by the server. Default: 10"
    }
];

export function registerMinecraftPropertiesLanguage(monaco: any)
{

    monaco.languages.register({id: "minecraft-properties"});
    monaco.languages.setLanguageConfiguration("minecraft-properties", minecraftPropertiesConfig);
    monaco.languages.setMonarchTokensProvider("minecraft-properties", minecraftPropertiesLanguage);

    monaco.languages.registerCompletionItemProvider("minecraft-properties", {
        provideCompletionItems: (model: any, position: any) =>
        {
            const word = model.getWordUntilPosition(position);
            const range = {
                startLineNumber: position.lineNumber,
                endLineNumber: position.lineNumber,
                startColumn: word.startColumn,
                endColumn: word.endColumn
            };

            return {
                suggestions: minecraftPropertiesCompletionItems.map(item => ({
                    ...item,
                    range
                }))
            };
        }
    });

    // Define custom theme for Minecraft properties
    monaco.editor.defineTheme("obsidian-editor-theme", {
        base: "vs-dark",
        inherit: true,
        rules: [
            {token: "key", foreground: "#46da84"},
            {token: "value", foreground: "#CE9178"},
            {token: "comment", foreground: "#393f49", fontStyle: "italic"},
            {token: "operator", foreground: "#D4D4D4"}
        ],
        colors: {
            "editor.background": "#18181b",
        }
    });
}
