import { createApp } from 'vue'
import { createPinia } from 'pinia'
import '@fontsource-variable/bricolage-grotesque/wght.css'
import './styles/base.css'
import App from './App.vue'
import { router } from './router'

// 服务端为搜索引擎预渲染的内容放在 #app 里，Vue 挂载时整体替换。
createApp(App).use(createPinia()).use(router).mount('#app')
