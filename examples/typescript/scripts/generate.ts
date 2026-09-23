// Programmatic usage: no config file is read, the object is the whole config.
import { fileURLToPath } from 'node:url'
import { generate } from '@denkwerk/favicon-generator'

await generate(
  {
    input: 'assets/favicon.svg',
    output: 'public/favicons-api',
    overwrite: true,
    pathPrefix: '/favicons-api/',
    appName: 'Favicon Generator Demo',
    snippets: ['html'],
  },
  { cwd: fileURLToPath(new URL('..', import.meta.url)) },
)
