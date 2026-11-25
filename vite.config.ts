import {createHash} from "node:crypto";
import {readdirSync, readFileSync} from "node:fs";
import {defineConfig} from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const meshSources = new URL("./src/components/worldmap/structure/", import.meta.url);

function meshSourceHash(): string {
    const hash = createHash("sha256");
    for (const name of readdirSync(meshSources).filter((file) => file.endsWith(".ts")).sort()) {
        hash.update(name).update(readFileSync(new URL(name, meshSources)));
    }
    return hash.digest("hex").slice(0, 16);
}

export default defineConfig({
    plugins: [react(), tailwindcss()],
    // Evaluated when the config loads, so a running dev server keeps the old version until it restarts.
    define: {
        __MESH_VERSION__: JSON.stringify(meshSourceHash()),
    },
    clearScreen: false,
    server: {
        host: true,
        allowedHosts: ["70.8.194.110"],
        port: 3000,
        strictPort: true,
        hmr: {
            protocol: "ws",
            host: "localhost",
            port: 3000,
            clientPort: 3000,
            overlay: true
        },
        watch: {
            ignored: ["**/src-*/**"]
        }
    },
    build: {
        outDir: "target/wwwroot",
        rolldownOptions: {
            output: {
                codeSplitting: {
                    groups: [
                        {name: "heroui", test: /node_modules[\\/](@heroui|@react-aria|react-aria|@react-stately|@internationalized)/},
                        {name: "preload", test: /vite[\\/]preload-helper/, priority: 1},
                        {name: "monaco", test: /node_modules[\\/](monaco-editor|@monaco-editor)/},
                        {name: "vendor", test: /node_modules/}
                    ]
                }
            }
        }
    }
});
