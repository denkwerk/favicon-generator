# example-cli-figma

Like [`examples/cli`](../cli), but the [`favicon.config.ts`](favicon.config.ts) uses a Figma component as `input`.
The CLI exports it as SVG through the [Figma REST API](https://developers.figma.com/docs/rest-api/) and generates the
icons from it.

This needs a personal access token with the `file_content:read` scope (Figma → Settings → Security → Personal access
tokens). Put it into `.env.local` at the repository root (gitignored), starting from the committed
[`.env.example`](../../.env.example):

```bash
cp .env.example .env.local   # then set FIGMA_TOKEN=figd_…
pnpm --filter example-cli-figma generate
```

A `FIGMA_TOKEN` env var works too and takes precedence: `FIGMA_TOKEN=figd_… pnpm generate`.

Output goes to `public/favicons` (gitignored). CI only runs this example when a `FIGMA_TOKEN` secret is configured.
