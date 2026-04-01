import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { pinia } from './stores'
import i18n from './i18n'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import './config/fontawesome'
import './styles/global.css'

const app = createApp(App)

app.use(pinia)
app.use(i18n)
app.use(router)

app.component('font-awesome-icon', FontAwesomeIcon)

app.mount('#app')
