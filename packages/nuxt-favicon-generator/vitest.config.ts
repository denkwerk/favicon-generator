import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['test/**/*.test.ts'],
    // The e2e tests build a Nuxt app per fixture.
    testTimeout: 60_000,
    hookTimeout: 180_000,
  },
})
