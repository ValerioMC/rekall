import { createApp } from 'vue'
import { createPinia } from 'pinia'
import Bootstrap from './Bootstrap.vue'
import './assets/main.css'

// No router: the console is one surface, and what would have been a route is a selection.
// Bootstrap decides, once, whether that surface is the console, the first-run wizard or the
// unreachable-database recovery screen.
createApp(Bootstrap).use(createPinia()).mount('#app')
