import { html } from 'virtual:favicons'

// The same tags the plugin injected into index.html, e.g. for an SSR template.
document.querySelector('#tags')!.textContent = html
