import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './assets/main.css'

// No router: the console is one surface, and what would have been a route is a selection.
createApp(App).use(createPinia()).mount('#app')
