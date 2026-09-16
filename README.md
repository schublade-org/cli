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

## Static build

Write a self-contained HTML workshop that runs without the Rust server. Controls, theme, and a11y keep working — rendering happens in the browser.

```bash
cargo run -- build
cargo run -- build --out dist
cargo run -- build --config examples/story-files/schublade.toml --out dist/story-files
```

The folder contains `index.html` (catalog baked in), `preview.html`, CSS/JS (including React for JSX stories), `bootstrap.json`, and a small `vercel.json`. Hash routes (`#/button`) do not need SPA rewrites.

Deploy that folder to any static host — Vercel, Netlify, nginx, GitHub Pages, or `python -m http.server`. Create one Vercel project per example yourself and point the project at that example’s output folder. Schublade does not talk to Vercel.

```bash
bash examples/build-all.sh
# writes dist/examples/<name>/
```

## Story files

The source of truth for a component’s props is a prescribed file next to the component — Storybook-like CSF, not comments or AST extraction.

Define the actual component in HTML or React. The story file imports that component and declares `args` / `argTypes` there. There is no second copy of the markup and no `code = "<Button…>"` usage template.

The CLI discovers `*.stories.js` / `*.stories.jsx` first. `*.stories.toml` remains a fallback.

```jsx
// components/button.jsx
export function Button({ label, variant, disabled }) {
  return (
    <button className="btn" data-variant={variant} disabled={disabled}>
      {label}
    </button>
  );
}
```

```jsx
// components/button.stories.jsx
import { Button } from './button.jsx';

export default {
  title: 'Components/Button',
  component: Button,
  args: {
    label: 'Save changes',
    variant: 'primary',
    disabled: false,
  },
  argTypes: {
    variant: { control: 'select', options: ['primary', 'ghost'] },
    disabled: { control: 'boolean' },
  },
};

export const Default = {};

export const Ghost = {
  args: { variant: 'ghost', label: 'Cancel' },
};
```

HTML templates work the same way: import the file and point `component` at it.

```js
// components/badge.stories.js
import html from './badge.html';

export default {
  title: 'Components/Badge',
  component: html,
  args: { tone: 'neutral', label: 'In review' },
  argTypes: {
    tone: { control: 'select', options: ['neutral', 'accent', 'warning'] },
  },
};
```

`args` are defaults. `argTypes` declare controls (`text`, `select`, `boolean`, `number`). If `argTypes` is omitted, the CLI infers a control from each arg value. Named `export const` objects are variants; they inherit meta args. Code Usage is generated from the imported component name plus the current args.

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

Each example has a `run.sh` that does the same thing, and `build.sh` where a static export is useful. `--config` loads that folder’s `schublade.toml`; the `catalog` and `stories` paths in the file are resolved next to it.

The root Aarau catalog stays the default demo (`cargo run -- serve`). See [`examples/README.md`](examples/README.md).

## What you get

- **CLI + server** — `clap` + `axum`. The workshop UI is embedded in the binary. `schublade build` writes the same workshop as static HTML.
- **Story discovery** — `*.stories.jsx` / `*.stories.js` next to components (TOML still works), plus `catalog.toml` as fallback.
- **React preview** — `/api/render` returns the imported component source and current props. The iframe mounts React from vendored UMD plus a small local JSX transform. No npm toolchain in the consumer repo.
- **Isolated iframe preview** — the canvas is a `sandbox="allow-scripts"` iframe. The shell talks to it with `postMessage` only.
- **Native light/dark** — configurable trigger in `schublade.toml`: `data-attribute`, `class-name` / `className`, or `local-storage` / `localStorage`.
- **Native a11y** — a small DOM checker in the preview iframe. Enable, disable, or drop rules in `schublade.toml`.
- **Demo stories** — Accordion, Avatar group, Badge, Button, Chip.

## Configure

`schublade.toml` (loaded from the working directory, or `--config`):

```toml
catalog = "./catalog.toml"
stories = "./components"   # optional; walk for *.stories.js(x) / *.stories.toml

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
cargo run -- build --config examples/story-files/schublade.toml --out dist/story-files
```
