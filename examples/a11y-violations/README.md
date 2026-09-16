# A11y violations

Stories that fail the native iframe rules: missing `alt`, unnamed button, unnamed link, unlabeled field. There is also one passing button so you can compare a clean scan.

```bash
npx schublade serve --config ./schublade.toml
# or ./run.sh
```

http://127.0.0.1:47302 — open the Accessibility chip on each story.
