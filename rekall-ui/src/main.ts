import { createApp } from 'vue'
import { createPinia } from 'pinia'
import Bootstrap from './Bootstrap.vue'
import { router } from './router'
import './assets/main.css'

// Bootstrap decides, once, whether the screen is the console (still one surface, no route of
// its own worth naming), the catalog pages that sit beside it, the first-run wizard, or the
// unreachable-database recovery screen. Routing only ever applies to the first of those four.
createApp(Bootstrap).use(createPinia()).use(router).mount('#app')

// The one thing standing between the manifest below and an actual "Install Rekall" affordance
// in the browser's address bar. Registered after load so it never competes with the app itself
// for the first paint; see public/sw.js for why it is deliberately empty.
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    void navigator.serviceWorker.register('/sw.js')
  })
}
