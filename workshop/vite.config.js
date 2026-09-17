import react from "@vitejs/plugin-react";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import { workshopMockPlugin } from "./dev/plugin.js";

const workshopRoot = dirname(fileURLToPath(import.meta.url));

// Repo-author chrome loop only. `npm run build:workshop` still writes ui/chrome/
// via esbuild; `schublade serve` embeds that output and does not run Vite.
export default defineConfig({
  root: workshopRoot,
  publicDir: false,
  plugins: [react(), workshopMockPlugin()],
  server: {
    host: true,
    port: 5173,
    fs: {
      allow: [dirname(workshopRoot)],
    },
  },
});
