# Story files

CSF-style `*.stories.toml` files next to the real component templates. The root CLI walks `components/`, reads args/argTypes, and includes `./button.html` / `./badge.html`. `catalog.toml` is present as a name-only fallback.

```bash
./run.sh
# cargo run --manifest-path ../../Cargo.toml -- serve --config ./schublade.toml
```

http://127.0.0.1:47308
