# Story files

Storybook-like CSF next to the real components. The root CLI walks `components/`, imports the component from the story file, and builds controls from `args` / `argTypes`. Code Usage is generated from the imported name plus the current args — the story file does not restate the markup.

- `button.jsx` is a React function component. `button.stories.jsx` imports it.
- `badge.html` is an HTML template with `{{tokens}}`. `badge.stories.js` imports it.

`catalog.toml` is present as a name-only fallback.

```bash
npx schublade serve --config ./schublade.toml
npx schublade build --config ./schublade.toml --out ./dist
# or
./run.sh
./build.sh
```

`./run.sh` / `./build.sh` call `npx schublade` on the standalone mirror. In the monorepo they fall back to `cargo run --manifest-path ../../Cargo.toml`.

http://127.0.0.1:47308
