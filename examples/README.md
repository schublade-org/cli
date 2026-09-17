# Examples

Each folder is a real workshop: its own `schublade.toml` and, usually, `catalog.toml`.

On a standalone mirror (`schublade-org/examples-<name>`), use the published npm CLI — no Cargo, no monorepo:

```bash
npx schublade serve --config ./schublade.toml
npx schublade build --config ./schublade.toml --out ./dist
# or
./run.sh
```

Inside **schublade-org/schublade**, `./run.sh` / `./build.sh` fall back to `cargo run --manifest-path ../../Cargo.toml` so local CLI changes apply. You can still call the npm CLI from the repo root after `npm install`:

```bash
npx schublade serve --config examples/overflow/schublade.toml
npx schublade serve --config examples/story-files/schublade.toml
```

`--config` loads the example config. Paths in that file (`catalog`, `stories`) are resolved relative to the config.

`bash examples/build-all.sh` writes `dist/examples/<name>/` using the monorepo Cargo CLI. Point a Vercel (or any static) project at that folder.

CI mirrors each folder to `schublade-org/examples-<NAME>` (see the root README, **Example repo sync**). `examples/` here is the source of truth.

The default catalog at the repo root is unchanged. Use `npx schublade serve` or `cargo run -- serve` for that.

| Example | What it exercises | Port |
| --- | --- | --- |
| [empty-catalog](empty-catalog/) | Catalog with zero stories | 47301 |
| [a11y-violations](a11y-violations/) | image-alt, button-name, link-name, label | 47302 |
| [missing-accessible-names](missing-accessible-names/) | Empty and icon-only controls | 47303 |
| [theme-class-name](theme-class-name/) | Theme trigger `class-name` | 47304 |
| [theme-local-storage](theme-local-storage/) | Theme trigger `local-storage` | 47305 |
| [overflow](overflow/) | Avatar overflow and long copy | 47306 |
| [many-controls](many-controls/) | A story with a full control panel | 47307 |
| [story-files](story-files/) | CSF `*.stories.jsx` / `*.stories.js` next to HTML and React components | 47308 |
| [tailwind-tokens](tailwind-tokens/) | Tailwind `@theme` CSS token families (colors, `--text-*`, spacing, radius, shadow, …) | 47309 |
| [kitchen-sink](kitchen-sink/) | TypeScript prop metadata, CSF/TOML fallbacks, generated code, and passing/failing a11y fixtures | 47310 |
