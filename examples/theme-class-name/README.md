# Theme · class-name

Same workshop, different trigger. `schublade.toml` sets `trigger = "class-name"` so the preview iframe adds `light` or `dark` on `<html>` instead of `data-theme`.

```bash
npx schublade serve --config ./schublade.toml
# or ./run.sh
```

http://127.0.0.1:47304 — use the theme toggle in the canvas toolbar.
