<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import { tauri_init_note } from '../api/init'

const router = useRouter()
const mockPath = ref('')
const loading = ref(false)
const error = ref('')



const selectDirectory = async () => {
  error.value = ''

  try {
    const path = await open({
      directory: true,
      multiple: false,
    })

    if (typeof path === 'string') {
      mockPath.value = path
      return
    }

    error.value = '未选择目录'
  } catch (err) {
    console.error('选择目录失败:', err)
    error.value = '无法打开目录选择器'
  }
}


// 尝试设置一个默认路径（优先使用 Tauri 的 homeDir）
onMounted(async () => {
  try {
    const pkg = '@tauri-apps/api'
    const { homeDir } = await import((pkg as unknown as string) + '/path')
    const dir = await homeDir()
    if (dir) {
      mockPath.value = dir
      return
    }
  } catch (_err: unknown) {
    // 无法获取 homeDir：记录以便排查但不打断流程
    console.warn('homeDir lookup failed', _err)
  }
  // 浏览器或无法获取时，使用常见的 Windows 公共用户目录作为默认
  mockPath.value = 'C:\\Users\\Public'
})

const handleInit = async () => {
  if (!mockPath.value) return

  loading.value = true
  error.value = ''

  try {
    await tauri_init_note(mockPath.value)
    router.push('/review')
  } catch (err: unknown) {
    console.error('初始化失败:', err)
    error.value = err instanceof Error
      ? err.message
      : '初始化失败'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="init-container">
    <div class="init-card">
      <h1 class="title">欢迎使用错题本系统</h1>
      <p class="description">
        一个高效的个人知识管理工具，帮助您系统性地复习和掌握重点知识。
      </p>

      <div class="directory-section">
        <p class="section-label">第一步：选择资源存储目录</p>
        <div class="directory-input-group">
          <input
            v-model="mockPath"
            type="text"
            class="directory-input"
            placeholder="手动输入或点击按钮选择目录"
          />
          <button class="select-btn" @click="selectDirectory">选择目录</button>
        </div>
        <p v-if="mockPath" class="selected-path">
          已选择：{{ mockPath }}
        </p>
        <p v-if="error" class="error-msg">{{ error }}</p>
      </div>

      <button
        class="init-btn"
        :disabled="!mockPath || loading"
        :class="{ disabled: !mockPath || loading }"
        @click="handleInit"
      >
        {{ loading ? '初始化中...' : '初始化系统' }}
      </button>

    </div>
  </div>
</template>

<style scoped>
.init-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background-color: #f5f5f5;
  padding: 20px;
}

.init-card {
  background-color: #ffffff;
  border-radius: 16px;
  padding: 48px;
  width: 100%;
  max-width: 480px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.title {
  font-size: 28px;
  color: #333;
  margin-bottom: 16px;
  text-align: center;
}

.description {
  color: #666;
  line-height: 1.6;
  margin-bottom: 32px;
  text-align: center;
}

.directory-section {
  margin-bottom: 24px;
}

.section-label {
  color: #333;
  font-size: 14px;
  margin-bottom: 12px;
  font-weight: 500;
}

.directory-input-group {
  display: flex;
  gap: 12px;
}

.directory-input {
  flex: 1;
  padding: 12px 16px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: #fafafa;
  color: #333;
  font-size: 14px;
  cursor: pointer;
}

.directory-input:focus {
  outline: none;
  border-color: #4CAF50;
  background-color: #ffffff;
}

.select-btn {
  padding: 12px 20px;
  background-color: #fff;
  border: 1px solid #4CAF50;
  border-radius: 8px;
  color: #4CAF50;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.select-btn:hover {
  background-color: #4CAF50;
  color: #ffffff;
}

.selected-path {
  color: #4CAF50;
  font-size: 13px;
  margin-top: 8px;
}

.init-btn {
  width: 100%;
  padding: 14px;
  background-color: #4CAF50;
  border: none;
  border-radius: 8px;
  color: #ffffff;
  font-size: 16px;
  font-weight: 600;
  transition: all 0.2s;
}

.init-btn:hover:not(.disabled) {
  background-color: #45a049;
  transform: translateY(-2px);
}

.init-btn.disabled {
  background-color: #e0e0e0;
  color: #999;
  cursor: not-allowed;
}
</style>
