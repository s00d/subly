import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "path";
import { mkdirSync, readFileSync, writeFileSync } from "fs";
import type { Plugin } from "vite";

/** Dev server store for `src/dev/ipcFixtures.ts`: latest IPC response per command+args. */
function ipcFixtures(): Plugin {
  const file = resolve(__dirname, ".dev/ipc-fixtures.json");
  const load = (): Record<string, unknown> => {
    try {
      return JSON.parse(readFileSync(file, "utf8"));
    } catch {
      return {};
    }
  };
  return {
    name: "subly-ipc-fixtures",
    apply: "serve",
    configureServer(server) {
      server.middlewares.use("/__ipc-fixtures", (req, res) => {
        if (req.method === "POST") {
          let body = "";
          req.on("data", (c) => (body += c));
          req.on("end", () => {
            const f = JSON.parse(body);
            const all = load();
            all[`${f.cmd}:${JSON.stringify(f.args)}`] = f;
            mkdirSync(resolve(__dirname, ".dev"), { recursive: true });
            writeFileSync(file, JSON.stringify(all));
            res.end("ok");
          });
          return;
        }
        res.setHeader("Content-Type", "application/json");
        res.end(JSON.stringify(Object.values(load())));
      });
    },
  };
}

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue(), tailwindcss(), ipcFixtures()],

  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    chunkSizeWarningLimit: 700,
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        splashscreen: resolve(__dirname, "splashscreen.html"),
      },
      output: {
        manualChunks(id) {
          if (!id.includes("node_modules")) return undefined;
          if (id.includes("/vue/") || id.includes("/vue-router/") || id.includes("/pinia/")) {
            return "vendor-vue";
          }
          if (id.includes("echarts") || id.includes("vue-echarts")) {
            return "vendor-charts";
          }
          if (id.includes("@tauri-apps")) {
            return "vendor-tauri";
          }
          return "vendor";
        },
      },
    },
  },
}));
