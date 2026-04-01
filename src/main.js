import { createApp } from 'vue'
// Vuetify
import 'vuetify/styles'
import 'mana-font/css/mana.min.css'
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { aliases, mdi } from 'vuetify/iconsets/mdi-svg'

// components
import App from './App.vue'
import {router} from "./router/index.ts";

const vuetify = createVuetify({
    components,
    directives,
    theme: {
        defaultTheme: 'dark',
        themes: {
            dark: {
                colors: {
                    primary: '#2196F3', // A bright, standard blue
                    background: '#0f0f0f', // Very dark, matching the screenshot
                    surface: '#1e1e1e', // Slightly lighter for cards
                }
            }
        }
    },
    defaults: {
        VBtn: {
            variant: 'outlined',
            color: 'primary',
        },
    },
    icons: {
        defaultSet: 'mdi',
        aliases,
        sets: {
            mdi,
        },
    },
})

createApp(App).use(vuetify).use(router).mount('#app')
