# Examples

Each folder is a real workshop: its own `schublade.toml` and, usually, `catalog.toml`. From the example directory the scripts invoke the **repo-root CLI** — not a special demo mode.

```bash
cd examples/empty-catalog
cargo run --manifest-path ../../Cargo.toml -- serve --config ./schublade.toml
# or
./run.sh
```

`--config` loads the example config. Paths in that file (`catalog`, `stories`) are resolved relative to the config, so this also works from the repo root:

```bash
cargo run -- serve --config examples/overflow/schublade.toml
cargo run -- serve --config examples/story-files/schublade.toml
```

`--catalog` and `--stories` override the paths in the config if you need to point at a file or directory directly.

Build a portable static site for an example (no server at runtime):

```bash
cd examples/story-files
cargo run --manifest-path ../../Cargo.toml -- build --config ./schublade.toml --out ./dist
# or
./build.sh
```

`bash examples/build-all.sh` writes `dist/examples/<name>/` for every example. Point a Vercel (or any static) project at that folder.

The default Aarau catalog at the repo root is unchanged. Use `cargo run -- serve` for that.

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
