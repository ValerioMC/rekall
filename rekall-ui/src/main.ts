import { createApp } from 'vue'
import { createPinia } from 'pinia'
import Bootstrap from './Bootstrap.vue'
import { router } from './router'
import { guardStrayFileDrops } from './common/shell/stray-file-drop'
import './assets/main.css'

guardStrayFileDrops(window)

createApp(Bootstrap).use(createPinia()).use(router).mount('#app')

if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    void navigator.serviceWorker.register('/sw.js')
  })
}
