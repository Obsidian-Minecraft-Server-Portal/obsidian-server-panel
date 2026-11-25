import {createHash} from "node:crypto";
import {readdirSync, readFileSync} from "node:fs";
import {fileURLToPath} from "node:url";
import {type Plugin, defineConfig, normalizePath} from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const meshDir = normalizePath(fileURLToPath(new URL("./src/components/worldmap/structure/", import.meta.url)));
const cellMesherFile = `${meshDir}cell-mesher.ts`;

function meshSourceHash(): string {
    const hash = createHash("sha256");
    for (const name of readdirSync(meshDir).filter((file) => file.endsWith(".ts")).sort()) {
        hash.update(name).update(readFileSync(meshDir + name));
    }
    return hash.digest("hex").slice(0, 16);
}

function meshVersion(): Plugin {
    let version = meshSourceHash();
    return {
        name: "mesh-version",
        transform(code, id) {
            if (normalizePath(id) !== cellMesherFile) {
                return null;
            }
            return code.replace(/__MESH_VERSION__/g, JSON.stringify(version));
        },
        configureServer(server) {
            const onSourceChange = (file: string) => {
                const source = normalizePath(file);
                if (!source.startsWith(meshDir) || !source.endsWith(".ts")) {
                    return;
                }
                let next: string;
                try {
                    next = meshSourceHash();
                } catch {
                    // A save can remove a file mid-read; the next event recomputes.
                    return;
                }
                if (next === version) {
                    return;
                }
                version = next;
                server.moduleGraph.getModulesByFile(cellMesherFile)?.forEach((mod) => server.moduleGraph.invalidateModule(mod));
                // Open pages keep the version they loaded, so they reload to pick up the new one.
                server.ws.send({type: "full-reload"});
            };
            server.watcher.on("add", onSourceChange);
            server.watcher.on("change", onSourceChange);
            server.watcher.on("unlink", onSourceChange);
        },
    };
}

export default defineConfig({
    plugins: [react(), tailwindcss(), meshVersion()],
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
