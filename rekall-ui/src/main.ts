import { createApp } from 'vue'
import { createPinia } from 'pinia'
import Bootstrap from './Bootstrap.vue'
import { router } from './router'
import './assets/main.css'

createApp(Bootstrap).use(createPinia()).use(router).mount('#app')

// The one thing standing between the manifest below and an actual "Install Rekall" affordance
// in the browser's address bar. Registered after load so it never competes with the app itself
// for the first paint; see public/sw.js for why it is deliberately empty.
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    void navigator.serviceWorker.register('/sw.js')
  })
}
