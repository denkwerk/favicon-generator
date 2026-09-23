# example-cli-figma

Like [`examples/cli`](../cli), but the [`favicon.config.ts`](favicon.config.ts) uses a Figma component as `input`.
The CLI exports it as SVG through the [Figma REST API](https://developers.figma.com/docs/rest-api/) and generates the
icons from it.

This needs a personal access token with the `file_content:read` scope (Figma → Settings → Security → Personal access
tokens), in the `FIGMA_TOKEN` env var or in `.figma-token` at the repository root (gitignored):

```bash
FIGMA_TOKEN=figd_… pnpm generate
```

Output goes to `public/favicons` (gitignored). CI only runs this example when a `FIGMA_TOKEN` secret is configured.
