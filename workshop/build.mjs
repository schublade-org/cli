import { build } from "esbuild";
import { readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

// Repo-author compile. Vite (`npm run dev:workshop`) is the HMR loop only.
// This still writes committed ui/chrome/ that `schublade serve` embeds.
const outdir = "ui/chrome";

await build({
  entryPoints: [
    "workshop/main.jsx",
    "workshop/App.jsx",
    "workshop/ColorScales.jsx",
    "workshop/Controls.jsx",
    "workshop/DocsPage.jsx",
    "workshop/IconButton.jsx",
    "workshop/Inspector.jsx",
    "workshop/Sidebar.jsx",
    "workshop/TypographyDocs.jsx",
    "workshop/groups.js",
    "workshop/highlight.js",
    "workshop/icons.js",
    "workshop/tokens.js",
  ],
  outdir,
  bundle: false,
  format: "esm",
  jsx: "automatic",
  platform: "browser",
  legalComments: "none",
});

for (const name of await readdir(outdir)) {
  if (!name.endsWith(".js")) continue;
  const path = join(outdir, name);
  const source = await readFile(path, "utf8");
  await writeFile(path, source.replaceAll(".jsx\"", ".js\"").replaceAll(".jsx'", ".js'"));
}

console.log("wrote ui/chrome");
