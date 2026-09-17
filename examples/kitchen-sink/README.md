# Kitchen Sink

A single workshop for metadata, TypeScript props, story discovery, and accessibility edge cases.

It includes one card for each of these paths:

- `type Props` without comments
- `type Props` with a comment on every prop
- `interface Props` without comments
- `interface Props` with a comment on every prop
- TOML without an explicit `code` field
- CSF-only story metadata, with no TOML story entry
- many intentional accessibility errors
- a comparable story with no accessibility errors

The TypeScript fixtures also cover optional props, literal unions, runtime defaults, and declared-only props.

```bash
./run.sh
./build.sh
```

http://127.0.0.1:47310
