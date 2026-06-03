import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'
import { setInitialized } from './mock/data'
import { tauri_check_init_default } from './api/init'
import { useSettingsStore } from './stores/settings'

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);
app.use(router);
app.mount('#app');

// 启动后在 Tauri 环境中请求后端检查默认数据根是否已初始化
(async () => {
  const status = await tauri_check_init_default()
  setInitialized(status.initialized, status.root)
  if (status.initialized) {
    // 加载设置
    const settingsStore = useSettingsStore()
    await settingsStore.loadSettings()
    router.push('/review')
  } else {
    router.push('/init')
  }
})().catch(e => {
  // 仅在 Tauri 环境运行；若发生错误则抛出以便尽早发现问题
  console.error('tauri check init failed', e)
  throw e
})
