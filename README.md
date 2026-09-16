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
cargo run -- serve --config ./schublade.toml --stories ./components
```

`schublade` with no subcommand also starts the server.

## Story files

The source of truth for a component’s props is a prescribed file next to the component — the same idea as Storybook CSF, not comments or AST extraction.

Comments and JSDoc are too easy to drift. Pulling props out of arbitrary React/Vue/Svelte/HTML is incomplete (unions, defaults, re-exports). Those can be helpers later. The CLI discovers `*.stories.toml` (or `*.story.toml`) and serves them.

```toml
# components/button.stories.toml
title = "Components/Button"
description = "The primary action control."
component = "./button.html"
code = """<Button variant="{{variant}}">{{label}}</Button>"""

[args]
label = "Save changes"
variant = "primary"
disabled = false

[argTypes.variant]
control = "select"
name = "Variant"
options = [
  { value = "primary", label = "Primary" },
  { value = "ghost", label = "Ghost" },
]

[[stories]]
name = "Default"

[[stories]]
name = "Ghost"
[stories.args]
variant = "ghost"
```

`component` includes the actual template (relative to the story file). `args` are defaults. `argTypes` declare controls (`text`, `select`, `boolean`, `number`). If `argTypes` is omitted, the CLI infers a control from each arg value. Named `[[stories]]` blocks are CSF-style variants; they inherit meta args.

Point the root CLI at a folder:

```toml
# schublade.toml
name = "My kit"
stories = "./components"
catalog = "./catalog.toml"  # optional fallback
```

`catalog.toml` still works. Existing examples keep using it. When both are present, discovered story files merge in; a matching `id` from a story file replaces the catalog entry. See [`examples/story-files/`](examples/story-files/).

## Examples

Edge-case catalogs live under [`examples/`](examples/). They are real configs, not an in-root demo mode. From an example directory, invoke the **repo-root CLI**:

```bash
cd examples/empty-catalog
cargo run --manifest-path ../../Cargo.toml -- serve --config ./schublade.toml
```

Each example has a `run.sh` that does the same thing. `--config` loads that folder’s `schublade.toml`; the `catalog` and `stories` paths in the file are resolved next to it.

The root Aarau catalog stays the default demo (`cargo run -- serve`). See [`examples/README.md`](examples/README.md).

## What you get

- **CLI + server** — `clap` + `axum`. The workshop UI is embedded in the binary.
- **Story discovery** — `*.stories.toml` next to components, plus `catalog.toml` as fallback.
- **Isolated iframe preview** — the canvas is a `sandbox="allow-scripts"` iframe. The shell talks to it with `postMessage` only.
- **Native light/dark** — configurable trigger in `schublade.toml`: `data-attribute`, `class-name` / `className`, or `local-storage` / `localStorage`.
- **Native a11y** — a small DOM checker in the preview iframe. Enable, disable, or drop rules in `schublade.toml`.
- **Demo stories** — Accordion, Avatar group, Badge, Button, Chip.

## Configure

`schublade.toml` (loaded from the working directory, or `--config`):

```toml
catalog = "./catalog.toml"
stories = "./components"   # optional; walk for *.stories.toml

[theme]
trigger = "data-attribute" # or "class-name", "local-storage"
key = "data-theme"
light = "light"
dark = "dark"

[a11y]
enabled = true
rules = ["image-alt", "button-name", "link-name", "label", "control-name"]
```

A relative `catalog` or `stories` path is resolved against the config file’s directory. Restart the server after edits. The Avatar group story uses a built-in renderer; HTML stories interpolate `{{control}}` tokens.

An empty catalog (name only, no `[[stories]]` and no story files) is valid — the workshop shows an empty state instead of refusing to start.

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
cargo run -- serve --config examples/story-files/schublade.toml
```
