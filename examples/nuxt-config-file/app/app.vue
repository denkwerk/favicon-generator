<script setup lang="ts">
const sizes = [16, 32, 96, 192, 512]
const files = ['favicon.ico', 'favicon.svg', 'apple-touch-icon.png', 'manifest.json', 'browserconfig.xml']

// The module serves the files under app.baseURL.
const { baseURL } = useRuntimeConfig().app
const url = (file: string) => `${baseURL}${file}`
</script>

<template>
  <main>
    <h1>@denkwerk/nuxt-favicon-generator</h1>
    <p>
      The favicons below are generated at build time from the source configured in
      <code>favicon.config.ts</code>, and the matching <code>&lt;link&gt;</code> and
      <code>&lt;meta&gt;</code> tags are in this page's <code>&lt;head&gt;</code>.
    </p>
    <ul class="icons">
      <li v-for="size in sizes" :key="size">
        <img :src="url(`favicon-${size}x${size}.png`)" :width="Math.min(size, 128)" :height="Math.min(size, 128)" :alt="`${size}×${size}`">
        <span>{{ size }}×{{ size }}</span>
      </li>
    </ul>
    <p>Also generated:</p>
    <ul>
      <!-- `external` keeps the prerender crawler from treating these as pages. -->
      <li v-for="file in files" :key="file">
        <NuxtLink :to="url(file)" external>{{ file }}</NuxtLink>
      </li>
    </ul>
  </main>
</template>

<style>
body { font-family: system-ui, sans-serif; margin: 2rem; color: #1a1a1a; }
.icons { display: flex; flex-wrap: wrap; align-items: flex-end; gap: 1.5rem; padding: 0; list-style: none; }
.icons li { display: flex; flex-direction: column; align-items: center; gap: 0.5rem; font-size: 0.875rem; }
</style>
