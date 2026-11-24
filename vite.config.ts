import {defineConfig} from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
    plugins: [react(), tailwindcss()],
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
