import { createApp } from 'vue'
import { createPinia } from 'pinia'
import Bootstrap from './Bootstrap.vue'
import './assets/main.css'

// No router: the console is one surface, and what would have been a route is a selection.
// Bootstrap decides, once, whether that surface is the console, the first-run wizard or the
// unreachable-database recovery screen.
createApp(Bootstrap).use(createPinia()).mount('#app')

// The one thing standing between the manifest below and an actual "Install Rekall" affordance
// in the browser's address bar. Registered after load so it never competes with the app itself
// for the first paint; see public/sw.js for why it is deliberately empty.
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    void navigator.serviceWorker.register('/sw.js')
  })
}
