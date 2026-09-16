# Schublade

A Storybook-like component workshop that runs from a **Rust CLI**. Consumers start a local server; they do not add a JavaScript toolchain to their repository.

This is a POC/MVP. Feature parity with Storybook is not a goal. There is no plugin system.

The bundled demo catalog is **Aarau Designsystem**: left story nav, an isolated center canvas, right-hand controls, and Code Usage under the preview.

## Run

```bash
cargo run -- serve
```

The workshop listens on `http://127.0.0.1:47291` by default (`0.0.0.0:47291` so it is reachable in this environment).

```bash
cargo run -- serve --port 47291
cargo run -- serve --config ./schublade.toml
cargo run -- serve --config ./schublade.toml --catalog ./catalog.toml
```

`schublade` with no subcommand also starts the server.

## Examples

Edge-case catalogs live under [`examples/`](examples/). They are real configs, not an in-root demo mode. From an example directory, invoke the **repo-root CLI**:

```bash
cd examples/empty-catalog
cargo run --manifest-path ../../Cargo.toml -- serve --config ./schublade.toml
```

Each example has a `run.sh` that does the same thing. `--config` loads that folder’s `schublade.toml`; the `catalog` path in the file is resolved next to it. You can also pass `--catalog` to point at a file directly.

The root Aarau catalog stays the default demo (`cargo run -- serve`). See [`examples/README.md`](examples/README.md) for empty catalog, a11y violations, theme triggers, overflow, and many-controls.

## What you get

- **CLI + server** — `clap` + `axum`. The workshop UI is embedded in the binary.
- **Isolated iframe preview** — the canvas is a `sandbox="allow-scripts"` iframe. The shell talks to it with `postMessage` only.
- **Native light/dark** — configurable trigger in `schublade.toml`: `data-attribute`, `class-name` / `className`, or `local-storage` / `localStorage`.
- **Native a11y** — a small DOM checker in the preview iframe. Enable, disable, or drop rules in `schublade.toml`.
- **Demo stories** — Accordion, Avatar group, Badge, Button, Chip.

## Configure

`schublade.toml` (loaded from the working directory, or `--config`):

```toml
catalog = "./catalog.toml"

[theme]
trigger = "data-attribute" # or "class-name", "local-storage"
key = "data-theme"
light = "light"
dark = "dark"

[a11y]
enabled = true
rules = ["image-alt", "button-name", "link-name", "label", "control-name"]
```

Stories live in `catalog.toml`. A relative `catalog` path is resolved against the config file’s directory. Restart the server after edits. The Avatar group story uses a built-in renderer; the others are HTML templates with `{{control}}` tokens.

An empty catalog (name only, no `[[stories]]`) is valid — the workshop shows an empty state instead of refusing to start.

## Layout

| Region | Role |
| --- | --- |
| Left | Catalog + story list |
| Center | Sandboxed canvas, viewport switcher, theme, a11y status |
| Right | Controls for the selected story |
| Bottom | Code Usage for the current control values |

## Develop

Rust 1.85 or newer (`rust-toolchain.toml` pins 1.85.0).

```bash
cargo test
cargo run -- serve
```
