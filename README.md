# Schublade

A Storybook-like component workshop that runs from a **Rust CLI**. Consumers start a local server; they do not add a JavaScript toolchain to their repository.

This is a POC/MVP. Feature parity with Storybook is not a goal. There is no plugin system.

The bundled demo catalog is **Aarau Designsystem**: left story nav, an isolated center canvas, right-hand controls, and Code Usage under the preview.

## Install

The CLI is a Rust binary wrapped for npm. Consumers need **npm** (or another Node 18+ package runner). They do not add a JS toolchain, bundler, or Rust to the catalog repo.

```bash
npx schublade serve
npx schublade build
```

`npx` downloads the `schublade` package. `postinstall` fetches the matching prebuilt binary from the GitHub Release for this version (`schublade-vX.Y.Z-<rust-target>.tar.gz`). No compile step.

```bash
npm install -g schublade
schublade serve --config ./schublade.toml
```

Supported prebuilt targets: macOS (arm64, x64), Linux glibc (x64, arm64), Windows x64. Alpine/musl is not supported.

From this repository, `npm install` builds with Cargo when Release assets are missing (`SCHUBLADE_SKIP_DOWNLOAD=1` forces that path). `SCHUBLADE_BINARY=/path/to/schublade` overrides the downloaded binary.

## Run

```bash
npx schublade serve
npx schublade serve --port 47291
npx schublade serve --config ./schublade.toml
npx schublade serve --config ./schublade.toml --catalog ./catalog.toml
npx schublade serve --config ./schublade.toml --stories ./components
```

The workshop listens on `http://127.0.0.1:47291` by default (`0.0.0.0:47291` so it is reachable in this environment). `schublade` with no subcommand also starts the server.

`serve` watches the resolved `schublade.toml`, `catalog.toml`, and stories/component tree. Catalog and story discovery reload in place — the HTTP server stays up. The workshop UI picks up the new catalog over `/api/events` (or `/api/generation` if EventSource is missing) and re-renders the iframe. Changing bind host/port in the config does not rebind; restart for that.

When hacking on the CLI itself:

```bash
cargo run -- serve
cargo run -- serve --config ./schublade.toml
```

## Static build

Write a self-contained HTML workshop that runs without the Rust server. Controls, theme, and a11y keep working — rendering happens in the browser.

```bash
npx schublade build
npx schublade build --out dist
npx schublade build --config examples/story-files/schublade.toml --out dist/story-files
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

Edge-case catalogs live under [`examples/`](examples/). They are real configs, not an in-root demo mode.

```bash
cd examples/empty-catalog
npx schublade serve --config ./schublade.toml
# or
./run.sh
```

Each `run.sh` / `build.sh` uses **`npx schublade`** when the folder is a standalone checkout (the mirrored `schublade-org/examples-<name>` repos). Inside this monorepo they fall back to `cargo run --manifest-path ../../Cargo.toml` so local CLI changes apply. `--config` loads that folder’s `schublade.toml`; `catalog` and `stories` paths are resolved next to it.

The root Aarau catalog stays the default demo (`npx schublade serve` / `cargo run -- serve`). See [`examples/README.md`](examples/README.md).

## Example repo sync

`examples/` in **schublade-org/schublade** is the source of truth. On push to `main` (and via *Actions → Sync example repos*), [`.github/workflows/sync-examples.yml`](.github/workflows/sync-examples.yml) mirrors each folder:

| Folder | Mirror |
| --- | --- |
| `examples/story-files` | [schublade-org/examples-story-files](https://github.com/schublade-org/examples-story-files) |
| `examples/empty-catalog` | [schublade-org/examples-empty-catalog](https://github.com/schublade-org/examples-empty-catalog) |
| … | `schublade-org/examples-<NAME>` |

The job **creates** a public repo if it does not exist, then force-pushes the folder contents to `main`.

### Secret: `EXAMPLES_SYNC_TOKEN`

The workflow reads **only** the repository secret named `EXAMPLES_SYNC_TOKEN`. If that secret is missing or empty, the job **fails immediately** with an error that names the secret.

Do **not** store the PAT as `GITHUB_TOKEN`. GitHub Actions already injects an automatic token under that name; a custom secret called `GITHUB_TOKEN` is ignored, and the automatic token cannot create or push other repositories in `schublade-org`.

Create a personal access token (the token owner must be allowed to create repositories in `schublade-org`), then add it at **Settings → Secrets and variables → Actions → New repository secret** with the exact name `EXAMPLES_SYNC_TOKEN`.

**Classic PAT**

| Scope | Why |
| --- | --- |
| `repo` | Create repositories in the org and force-push `examples-*`. `public_repo` is not enough to create repositories. |

**Fine-grained PAT**

| Field | Value |
| --- | --- |
| Resource owner | `schublade-org` |
| Repository access | **All repositories** (must include future `examples-*` repos) |
| Administration | Read and write (create repos) |
| Contents | Read and write (push) |
| Metadata | Read (granted automatically) |

Org approval may be required before a fine-grained token can act on `schublade-org`.

Do not commit a token. This repository does not invent or ship credentials.

## Publish the npm package

Keep `package.json` and `Cargo.toml` versions in lockstep. Tag the same version:

```bash
git tag v0.1.0
git push origin v0.1.0
```

[`.github/workflows/release.yml`](.github/workflows/release.yml) builds platform archives, attaches them to a GitHub Release, and publishes to npm when **`NPM_TOKEN`** is set (npm automation token). If `NPM_TOKEN` is absent, the Release still goes up; publish later with `npm publish --access public` from the repo root.

The installer downloads `https://github.com/schublade-org/schublade/releases/download/v<version>/schublade-v<version>-<target>.tar.gz`.

## What you get

- **CLI + server** — `clap` + `axum`, published as the `schublade` npm package. The workshop UI is embedded in the binary. `npx schublade serve` hot-reloads catalog and stories; `npx schublade build` writes the same workshop as static HTML.
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

A relative `catalog` or `stories` path is resolved against the config file’s directory. `serve` reloads catalog, stories, and theme/a11y config in place; host/port changes still need a restart. The Avatar group story uses a built-in renderer; HTML stories interpolate `{{control}}` tokens.

An empty catalog (name only, no `[[stories]]` and no story files) is valid — the workshop shows an empty state instead of refusing to start.

## Layout

| Region | Role |
| --- | --- |
| Left | Catalog + story list |
| Center | Sandboxed canvas, viewport switcher, theme, a11y status |
| Right | Controls for the selected story |
| Bottom | Code Usage for the current control values |

## Develop

Rust 1.85 or newer (`rust-toolchain.toml` pins 1.85.0). Node 18+ is only required to exercise the npm wrapper.

```bash
cargo test
npm run test:npm
cargo run -- serve
cargo run -- serve --config examples/story-files/schublade.toml
cargo run -- build --config examples/story-files/schublade.toml --out dist/story-files
SCHUBLADE_SKIP_DOWNLOAD=1 npm install
npx schublade --help
```
