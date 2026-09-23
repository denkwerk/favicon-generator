# favicon-generator (crate)

The Rust CLI behind [`@denkwerk/favicon-generator`](../../packages/favicon-generator). See that README for all
options, config files and the Figma integration.

```bash
cargo run --release -- ../../examples/typescript/assets/favicon.svg out -p /public -n "Motel One"
cargo run --release -- --help
```

| Module | Responsibility |
| --- | --- |
| `cli.rs` | Command-line flags |
| `config.rs` | `favicon.config.*` discovery and loading (JS/TS configs are evaluated with Node.js; `FAVICON_GENERATOR_NODE` selects the binary), and merging with the flags |
| `figma.rs` | Figma REST API export |
| `source.rs` | SVG (resvg) and raster rendering |
| `spec.rs` | Generated sizes and file names |
| `assets.rs` | Manifest, browserconfig and `<head>` snippets |
