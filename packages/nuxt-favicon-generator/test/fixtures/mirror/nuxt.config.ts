import FaviconModule from '../../../src/module'

// Links the files under /public/ (for a CDN cache rule) and serves them at the
// site root as well, as a site that moved its favicons would.
export default defineNuxtConfig({
  modules: [FaviconModule],
  favicon: {
    input: './logo.svg',
    pathPrefix: '/public/',
    mirrorPrefixes: ['/'],
    legacy: true,
    manifest: { name: 'Mirror Fixture', iconSizes: [72, 192, 512], iconPurpose: 'any maskable' },
  },
  compatibilityDate: '2026-09-01',
})
