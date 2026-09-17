# Tailwind tokens

Adapter 4 reads a Tailwind `@theme` CSS file. This is not a generic custom-property walk — only known Tailwind prefixes become docs tokens.

`--text-*` is one family among several. It is overloaded (size vs color vs align/wrap skip vs `--text-shadow*`). Font size is `--text-*`; there is no `--font-size-*` prefix.

| Prefix | Family |
| --- | --- |
| `--color-*` | colors |
| `--text-*` | typography size, text color, or skip |
| `--text-*--line-height` | pairs with a `--text-*` size |
| `--text-shadow*` | text-shadow |
| `--font-*` | font family |
| `--font-weight-*` | font-weight |
| `--leading-*` | line-height |
| `--tracking-*` | letter-spacing |
| `--tab-size-*` | tab-size |
| `--spacing*` | spacing |
| `--radius*` | radius |
| `--shadow-*` | box-shadow |
| `--inset-shadow-*` | inset-shadow |
| `--drop-shadow-*` | drop-shadow |
| `--blur-*` | blur |
| `--perspective-*` | perspective |
| `--zoom-*` | zoom |
| `--aspect-*` | aspect-ratio |
| `--ease-*` | easing |
| `--animate-*` | animation |
| `--breakpoint-*` | breakpoints |
| `--container-*` | containers |

```bash
npx schublade serve --config ./schublade.toml
npx schublade build --config ./schublade.toml --out ./dist
# or
./run.sh
./build.sh
```

`./run.sh` / `./build.sh` call `npx schublade` on the standalone mirror. In the monorepo they fall back to `cargo run --manifest-path ../../Cargo.toml`.

http://127.0.0.1:47309
