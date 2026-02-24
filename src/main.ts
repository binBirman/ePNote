import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router'
import { setInitialized } from './mock/data'

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')

  // 启动后在 Tauri 环境中请求后端检查默认数据根是否已初始化
  ; (async () => {
    try {
      const { invoke } = await import('@tauri-apps/api' + '/core')
      const status = await invoke('tauri_check_init_default') as { initialized: boolean; root?: string }
      setInitialized(status.initialized, status.root)
      if (status.initialized) {
        // 已初始化：跳转到 review 主页面
        router.push('/review')
      } else {
        router.push('/init')
      }
    } catch (e) {
      // 非 Tauri 环境或错误：保留 mock 行为（默认路由 /init）
      // 可选择打印错误以便调试
      console.warn('check init failed', e)
    }
  })()
