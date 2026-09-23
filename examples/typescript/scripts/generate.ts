// Programmatic usage: no config file is read, the object is the whole config.
import { fileURLToPath } from 'node:url'
import { generate } from '@denkwerk/favicon-generator'

await generate(
  {
    input: 'assets/favicon.svg',
    output: 'public/favicons',
    overwrite: true,
    pathPrefix: '/favicons/',
    themeColor: '#02969c',
    manifest: { name: 'Favicon Generator TypeScript Example' },
    snippets: ['html'],
  },
  { cwd: fileURLToPath(new URL('..', import.meta.url)) },
)
