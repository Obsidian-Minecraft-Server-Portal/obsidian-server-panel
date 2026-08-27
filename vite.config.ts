import {defineConfig} from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import {fileURLToPath} from "node:url";

const port = Number(process.env.OBSIDIAN_VITE_PORT ?? 3000);

export default defineConfig({
    plugins: [react(), tailwindcss()],
    resolve: {
        alias: {
            "@heroui-compat": fileURLToPath(new URL("./src/components/heroui/index.ts", import.meta.url))
        }
    },
    css: {
        postcss: {}
    },
    esbuild: {
        legalComments: "none",
        supported: {
            "top-level-await": true // browsers can handle top-level-await features
        }
    },
    clearScreen: false,
    server: {
        host: true,
        port,
        strictPort: true,
        hmr: {
            protocol: "ws",
            host: "localhost",
            port,
            clientPort: port,
            overlay: true
        },
        watch: {
            ignored: ["**/src-*/**"]
        }
    },
    build: {
        outDir: "target/wwwroot"
    }
});