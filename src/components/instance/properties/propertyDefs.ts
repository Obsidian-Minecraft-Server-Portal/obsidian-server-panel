export type PropertyDef =
  | { key: string; label: string; hint: string; kind: "text" | "number"; readOnly?: boolean }
  | { key: string; label: string; hint: string; kind: "bool" }
  | { key: string; label: string; hint: string; kind: "select"; options: readonly string[] };

export interface PropertySection {
  id: string;
  label: string;
  properties: readonly PropertyDef[];
}

export const PROPERTY_SECTIONS: readonly PropertySection[] = [
  {
    id: "general",
    label: "General",
    properties: [
      { key: "motd", label: "Message of the day", hint: "Shown under the server name in the server list.", kind: "text" },
      { key: "max-players", label: "Max players", hint: "Maximum number of players that can join at once.", kind: "number" },
      { key: "server-port", label: "Server port", hint: "Managed by the instance port in Settings.", kind: "number", readOnly: true },
      { key: "view-distance", label: "View distance", hint: "Chunks sent to each player, from 3 to 32.", kind: "number" },
      { key: "simulation-distance", label: "Simulation distance", hint: "Chunks around each player that tick.", kind: "number" },
    ],
  },
  {
    id: "gameplay",
    label: "Gameplay",
    properties: [
      { key: "gamemode", label: "Default game mode", hint: "Mode given to players when they first join.", kind: "select", options: ["survival", "creative", "adventure", "spectator"] },
      { key: "difficulty", label: "Difficulty", hint: "Changes mob strength and hunger.", kind: "select", options: ["peaceful", "easy", "normal", "hard"] },
      { key: "pvp", label: "PvP", hint: "Players can damage each other.", kind: "bool" },
      { key: "hardcore", label: "Hardcore", hint: "Players are banned on death and difficulty is locked.", kind: "bool" },
      { key: "allow-flight", label: "Allow flight", hint: "Stops kicks for flying in survival.", kind: "bool" },
      { key: "allow-nether", label: "Allow the Nether", hint: "Lets players travel to the Nether.", kind: "bool" },
      { key: "enable-command-block", label: "Command blocks", hint: "Allows command blocks to run commands.", kind: "bool" },
      { key: "spawn-protection", label: "Spawn protection", hint: "Radius around spawn that only operators can edit.", kind: "number" },
    ],
  },
  {
    id: "world",
    label: "World",
    properties: [
      { key: "level-name", label: "World folder", hint: "Directory the world is saved in.", kind: "text" },
      { key: "level-seed", label: "Seed", hint: "Used when the world is first generated.", kind: "text" },
    ],
  },
  {
    id: "access",
    label: "Access & security",
    properties: [
      { key: "online-mode", label: "Require Mojang accounts", hint: "Turn off only behind a proxy that authenticates players.", kind: "bool" },
      { key: "white-list", label: "Whitelist", hint: "Only whitelisted players can join.", kind: "bool" },
      { key: "enforce-whitelist", label: "Enforce whitelist", hint: "Kick non-whitelisted players when the list changes.", kind: "bool" },
    ],
  },
];
