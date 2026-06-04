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
  await store.loadSubjectPool()
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

      <!-- 每科每日推荐题数 -->
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">每科每日推荐题数</span>
          <span class="setting-desc">每个科目每天推荐的题目数量</span>
        </div>
        <div class="setting-control">
          <div class="number-input-group">
            <button class="num-btn" @click="store.perSubjectDailyLimit = Math.max(1, store.perSubjectDailyLimit - 5)">-5</button>
            <button class="num-btn" @click="store.perSubjectDailyLimit = Math.max(1, store.perSubjectDailyLimit - 1)">-</button>
            <input
              v-model.number="store.perSubjectDailyLimit"
              type="number"
              class="num-input"
              min="1"
              max="50"
            />
            <button class="num-btn" @click="store.perSubjectDailyLimit = Math.min(50, store.perSubjectDailyLimit + 1)">+</button>
            <button class="num-btn" @click="store.perSubjectDailyLimit = Math.min(50, store.perSubjectDailyLimit + 5)">+5</button>
          </div>
        </div>
      </div>

      <!-- 新题保送比例 -->
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">新题保送比例</span>
          <span class="setting-desc">每天至少保证多少比例的名额给从未复习过的新题</span>
        </div>
        <div class="setting-control">
          <input
            v-model.number="store.newQuestionGuaranteeRatio"
            type="range"
            class="slider-input"
            min="0"
            max="1"
            step="0.05"
          />
          <span class="slider-value">{{ (store.newQuestionGuaranteeRatio * 100).toFixed(0) }}%</span>
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

        <!-- 开发者模式 -->
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">开发者模式</span>
            <span class="setting-desc">启用开发者中心，可查看推荐详情和调试功能</span>
          </div>
          <div class="setting-control">
            <button
              class="toggle-btn"
              :class="{ active: store.developerMode }"
              @click="store.developerMode = !store.developerMode"
            >
              {{ store.developerMode ? '开' : '关' }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 开发者入口 -->
    <div v-if="store.developerMode" class="settings-card dev-entry-card">
      <h2 class="card-title">开发者</h2>
      <p class="dev-desc">推荐算法调试与预览工具</p>
      <router-link to="/dev/center" class="dev-link">进入开发者中心</router-link>
    </div>

    <!-- 科目池管理 -->
    <div class="settings-card">
      <h2 class="card-title">科目池管理</h2>
      <p class="pool-desc">勾选的科目参与每日推荐，取消勾选归档该科目</p>

      <!-- 正常科目 -->
      <div v-for="subject in store.allSubjects" :key="subject" class="subject-row">
        <input type="checkbox"
          :checked="!store.subjectConfigs[subject]?.archived"
          @change="store.toggleSubjectArchive(subject)" />
        <span>{{ subject }}</span>
      </div>

      <!-- 未分类题目（虚拟科目，不在 subjectConfigs 中） -->
      <div class="subject-row unclassified">
        <input type="checkbox" checked disabled />
        <span>未分类题目</span>
        <span class="unclassified-hint">未标注科目的题目，始终参与推荐</span>
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

/* 科目池管理 */
.pool-desc {
  font-size: 13px;
  color: #999;
  margin-bottom: 16px;
}

.subject-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 0;
  border-bottom: 1px solid #f5f5f5;
  font-size: 15px;
  color: #333;
}

.subject-row:last-of-type {
  border-bottom: none;
}

.subject-row input[type="checkbox"] {
  width: 18px;
  height: 18px;
  accent-color: #4CAF50;
  cursor: pointer;
  flex-shrink: 0;
}

.subject-row.unclassified {
  opacity: 0.7;
}

.unclassified-hint {
  font-size: 12px;
  color: #aaa;
  margin-left: auto;
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

/* 开发者入口 */
.dev-entry-card {
  border: 1px solid #e0e0e0;
}

.dev-desc {
  font-size: 13px;
  color: #888;
  margin-bottom: 16px;
}

.dev-link {
  display: inline-block;
  padding: 10px 28px;
  background-color: #333;
  color: #fff;
  text-decoration: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.dev-link:hover {
  background-color: #555;
}
</style>
