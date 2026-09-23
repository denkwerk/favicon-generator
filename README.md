# favicon-generator

Generates a complete favicon set from a single SVG (recommended) or PNG/JPEG/WebP image.

```bash
cargo run --release -- favicon.svg out -p /public -n "Motel One" --manifest-crossorigin use-credentials
```

### From Figma

Pass a Figma link that includes a `node-id` instead of a file. The node is exported as SVG via the
[REST API](https://developers.figma.com/docs/rest-api/) (`GET /v1/images/:key?format=svg`) and used as the source.

```bash
echo "figd_…" > .figma-token   # gitignored
cargo run --release -- "https://www.figma.com/design/77SgSAGXbDv1Eye6htYdCG/ONE---Assets-Library-NEW?node-id=19938-42" out \
  --figma-token-file .figma-token -p /public -n "Motel One"
```

You need a personal access token with the `file_content:read` scope (Figma → Settings → Security → Personal access tokens).
It is read from `--figma-token`, the `FIGMA_TOKEN` env var, `--figma-token-file`, or the `FIGMA_TOKEN_FILE` env var.

## Output

| File | Notes |
| --- | --- |
| `favicon-{16,32,57,60,70,72,76,96,114,120,128,144,150,152,180,192,310,384,512}.png` | transparent RGBA |
| `apple-touch-icon.png`, `apple-touch-icon-{120x120,152x152}.png` (+ `-precomposed`) | opaque, flattened onto `--background-color` |
| `favicon.ico` | 16, 24, 32, 48, 64, 128, 256 |
| `favicon.svg` | copy of the input (SVG input only) |
| `manifest.json` | web app manifest |
| `browserconfig.xml` | Windows tiles |
| `favicon.html` / `nuxt-head.ts` | `<head>` tags as HTML, or as a TS object for Nuxt's `app.head` (`--snippets`) |

SVG sources are rendered natively at each size with [resvg](https://github.com/linebender/resvg);
raster sources are downscaled with Lanczos3. Run with `--help` for all options.
