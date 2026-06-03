<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'

const store = useSettingsStore()

const saving = ref(false)
const saveMessage = ref('')
const showAdvanced = ref(false)

onMounted(async () => {
  if (!store.loaded) {
    await store.loadSettings()
  }
})

async function handleSave() {
  saving.value = true
  saveMessage.value = ''
  try {
    await store.saveSettings()
    saveMessage.value = '设置已保存'
    setTimeout(() => { saveMessage.value = '' }, 3000)
  } catch (e) {
    saveMessage.value = '保存失败: ' + String(e)
  } finally {
    saving.value = false
  }
}

async function copyDataPath() {
  try {
    await navigator.clipboard.writeText(store.dataRoot)
    saveMessage.value = '路径已复制'
    setTimeout(() => { saveMessage.value = '' }, 2000)
  } catch {
    saveMessage.value = '复制失败'
  }
}
</script>

<template>
  <div class="settings-container">
    <h1 class="page-title">设置</h1>

    <!-- 复习设置 -->
    <div class="settings-card">
      <h2 class="card-title">复习设置</h2>

      <!-- 默认复习题数 -->
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">默认复习题数</span>
          <span class="setting-desc">进入复习页时的默认单次题目数量</span>
        </div>
        <div class="setting-control">
          <div class="number-input-group">
            <button class="num-btn" @click="store.defaultReviewLimit = Math.max(1, store.defaultReviewLimit - 5)">-5</button>
            <button class="num-btn" @click="store.defaultReviewLimit = Math.max(1, store.defaultReviewLimit - 1)">-</button>
            <input
              v-model.number="store.defaultReviewLimit"
              type="number"
              class="num-input"
              min="1"
              max="100"
            />
            <button class="num-btn" @click="store.defaultReviewLimit = Math.min(100, store.defaultReviewLimit + 1)">+</button>
            <button class="num-btn" @click="store.defaultReviewLimit = Math.min(100, store.defaultReviewLimit + 5)">+5</button>
          </div>
        </div>
      </div>

      <!-- 每日推荐上限 -->
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">每日推荐上限</span>
          <span class="setting-desc">每日推荐系统生成的最大题目总数</span>
        </div>
        <div class="setting-control">
          <div class="number-input-group">
            <button class="num-btn" @click="store.dailyRecommendationLimit = Math.max(1, store.dailyRecommendationLimit - 5)">-5</button>
            <button class="num-btn" @click="store.dailyRecommendationLimit = Math.max(1, store.dailyRecommendationLimit - 1)">-</button>
            <input
              v-model.number="store.dailyRecommendationLimit"
              type="number"
              class="num-input"
              min="1"
              max="200"
            />
            <button class="num-btn" @click="store.dailyRecommendationLimit = Math.min(200, store.dailyRecommendationLimit + 1)">+</button>
            <button class="num-btn" @click="store.dailyRecommendationLimit = Math.min(200, store.dailyRecommendationLimit + 5)">+5</button>
          </div>
        </div>
      </div>

      <!-- 新题推荐比例（预留） -->
      <div class="setting-row disabled">
        <div class="setting-info">
          <span class="setting-label">新题推荐比例</span>
          <span class="setting-desc">推荐中新题所占的比例（后续开放）</span>
        </div>
        <div class="setting-control">
          <input
            v-model.number="store.newQuestionRatio"
            type="range"
            class="slider-input"
            min="0"
            max="1"
            step="0.1"
            disabled
          />
          <span class="slider-value disabled-text">{{ store.newQuestionRatio.toFixed(1) }}</span>
        </div>
      </div>

      <!-- 高级设置折叠 -->
      <div class="advanced-toggle" @click="showAdvanced = !showAdvanced">
        <span class="toggle-arrow">{{ showAdvanced ? '▼' : '▶' }}</span>
        <span class="toggle-label">高级设置</span>
      </div>

      <div v-if="showAdvanced" class="advanced-section">
        <!-- 推荐随机性 -->
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">推荐随机性</span>
            <span class="setting-desc">推荐评分中随机因子的大小，值越大排序越随机</span>
          </div>
          <div class="setting-control">
            <input
              v-model.number="store.recommendationRandomness"
              type="range"
              class="slider-input"
              min="0.5"
              max="2.0"
              step="0.1"
            />
            <span class="slider-value">{{ store.recommendationRandomness.toFixed(1) }}</span>
          </div>
        </div>

        <!-- 显示推荐调试信息 -->
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">显示推荐调试信息</span>
            <span class="setting-desc">在推荐列表中展示评分详情和推荐理由（开发用）</span>
          </div>
          <div class="setting-control">
            <button
              class="toggle-btn"
              :class="{ active: store.showDebugInfo }"
              @click="store.showDebugInfo = !store.showDebugInfo"
            >
              {{ store.showDebugInfo ? '开' : '关' }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 数据管理 -->
    <div class="settings-card">
      <h2 class="card-title">数据管理</h2>

      <!-- 当前数据目录 -->
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">当前数据目录</span>
          <span class="setting-desc">所有错题数据的存储位置</span>
        </div>
        <div class="setting-control">
          <span class="data-path">{{ store.dataRoot || '未设置' }}</span>
        </div>
      </div>

      <div class="data-actions">
        <button class="data-btn" @click="copyDataPath">复制路径</button>
        <button class="data-btn primary" @click="store.openDataDir()">打开数据目录</button>
      </div>

      <!-- 导入导出（预留） -->
      <div class="setting-row disabled">
        <div class="setting-info">
          <span class="setting-label">导出数据</span>
          <span class="setting-desc">将所有题目导出为文件（后续开放）</span>
        </div>
        <div class="setting-control">
          <button class="data-btn" disabled>导出</button>
        </div>
      </div>

      <div class="setting-row disabled">
        <div class="setting-info">
          <span class="setting-label">导入数据</span>
          <span class="setting-desc">从文件导入题目数据（后续开放）</span>
        </div>
        <div class="setting-control">
          <button class="data-btn" disabled>导入</button>
        </div>
      </div>
    </div>

    <!-- 保存按钮 -->
    <div class="save-area">
      <button class="save-btn" :disabled="saving" @click="handleSave">
        {{ saving ? '保存中...' : '保存设置' }}
      </button>
      <span v-if="saveMessage" class="save-message">{{ saveMessage }}</span>
    </div>
  </div>
</template>

<style scoped>
.settings-container {
  max-width: 700px;
  width: 100%;
  padding-bottom: 40px;
}

.page-title {
  font-size: 32px;
  color: #333;
  margin-bottom: 30px;
  text-align: center;
}

.settings-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 24px 28px;
  margin-bottom: 20px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.card-title {
  font-size: 18px;
  font-weight: 600;
  color: #333;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid #eee;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 0;
  border-bottom: 1px solid #f5f5f5;
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-row.disabled {
  opacity: 0.5;
}

.setting-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.setting-label {
  font-size: 15px;
  font-weight: 500;
  color: #333;
}

.setting-desc {
  font-size: 12px;
  color: #999;
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

/* 数字输入组 */
.number-input-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.num-btn {
  padding: 6px 10px;
  background-color: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 4px;
  color: #333;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.num-btn:hover {
  background-color: #e8e8e8;
}

.num-input {
  width: 60px;
  padding: 6px;
  text-align: center;
  font-size: 15px;
  font-weight: 600;
  border: 2px solid #4CAF50;
  border-radius: 6px;
  color: #333;
}

.num-input:focus {
  outline: none;
  border-color: #45a049;
}

/* 滑块 */
.slider-input {
  width: 120px;
  accent-color: #4CAF50;
}

.slider-input:disabled {
  accent-color: #ccc;
}

.slider-value {
  font-size: 14px;
  font-weight: 600;
  color: #333;
  min-width: 30px;
  text-align: center;
}

.disabled-text {
  color: #999;
}

/* 开关按钮 */
.toggle-btn {
  padding: 6px 20px;
  background-color: #ccc;
  border: none;
  border-radius: 16px;
  color: #fff;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  min-width: 48px;
}

.toggle-btn.active {
  background-color: #4CAF50;
}

/* 高级设置 */
.advanced-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 0;
  cursor: pointer;
  user-select: none;
  border-top: 1px solid #f5f5f5;
}

.toggle-arrow {
  font-size: 12px;
  color: #666;
}

.toggle-label {
  font-size: 14px;
  color: #666;
  font-weight: 500;
}

.advanced-toggle:hover .toggle-label {
  color: #4CAF50;
}

.advanced-section {
  padding-left: 8px;
  border-left: 2px solid #4CAF50;
  margin-bottom: 4px;
}

/* 数据管理 */
.data-path {
  font-size: 13px;
  color: #666;
  background-color: #fafafa;
  padding: 6px 10px;
  border-radius: 4px;
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.data-actions {
  display: flex;
  gap: 10px;
  padding: 8px 0 18px;
}

.data-btn {
  padding: 8px 18px;
  background-color: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 6px;
  color: #333;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.data-btn:hover {
  background-color: #e8e8e8;
}

.data-btn.primary {
  background-color: #4CAF50;
  border-color: #4CAF50;
  color: #fff;
}

.data-btn.primary:hover {
  background-color: #45a049;
}

.data-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 保存区域 */
.save-area {
  display: flex;
  align-items: center;
  gap: 16px;
  justify-content: center;
  margin-top: 12px;
}

.save-btn {
  padding: 14px 48px;
  background-color: #4CAF50;
  border: none;
  border-radius: 10px;
  color: #ffffff;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.save-btn:hover {
  background-color: #45a049;
}

.save-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.save-message {
  font-size: 14px;
  color: #4CAF50;
}
</style>
