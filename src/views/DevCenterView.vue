<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import { getRecommendationStats, regenerateDailyRecommendation } from '@/api/review'
import type { RecommendationStats } from '@/api/review'

const router = useRouter()
const store = useSettingsStore()

const stats = ref<RecommendationStats | null>(null)
const loading = ref(true)
const statsError = ref('')

// 重新生成相关
const showRegenModal = ref(false)
const regenerating = ref(false)
const regenResult = ref('')

onMounted(async () => {
  await loadStats()
})

async function loadStats() {
  loading.value = true
  statsError.value = ''
  try {
    stats.value = await getRecommendationStats()
  } catch (e) {
    statsError.value = '加载统计失败: ' + String(e)
  } finally {
    loading.value = false
  }
}

function openRegenModal() {
  regenResult.value = ''
  showRegenModal.value = true
}

function cancelRegen() {
  showRegenModal.value = false
}

async function confirmRegen() {
  showRegenModal.value = false
  regenerating.value = true
  regenResult.value = ''
  try {
    await regenerateDailyRecommendation()
    regenResult.value = '重新生成完成'
    await loadStats()
  } catch (e) {
    regenResult.value = '重新生成失败: ' + String(e)
  } finally {
    regenerating.value = false
    setTimeout(() => { regenResult.value = '' }, 5000)
  }
}

function goToList() {
  router.push('/dev/recommendation-list')
}

function goToPreview() {
  router.push('/dev/preview-recommendation')
}

function goBack() {
  router.push('/settings')
}
</script>

<template>
  <div class="dev-center">
    <div class="page-header">
      <h1 class="page-title">开发者中心</h1>
      <button class="back-btn" @click="goBack">返回设置</button>
    </div>

    <!-- 统计面板 -->
    <div class="stats-card">
      <h2 class="card-title">推荐统计</h2>

      <div v-if="loading" class="loading-text">加载中...</div>
      <div v-else-if="statsError" class="error-text">{{ statsError }}</div>

      <div v-else-if="stats" class="stats-grid">
        <div class="stat-item">
          <span class="stat-value">{{ stats.total_questions }}</span>
          <span class="stat-label">总题数</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.participating_questions }}</span>
          <span class="stat-label">参与推荐</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.archived_subjects.length }}</span>
          <span class="stat-label">已归档科目</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.recommended_count }}</span>
          <span class="stat-label">今日推荐</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.new_questions }}</span>
          <span class="stat-label">新题</span>
        </div>
        <div class="stat-item">
          <span class="stat-value">{{ stats.pending_review }}</span>
          <span class="stat-label">待复习</span>
        </div>
      </div>

      <div v-if="stats && stats.archived_subjects.length > 0" class="archived-list">
        <span class="archived-label">已归档科目：</span>
        <span class="archived-names">{{ stats.archived_subjects.join('、') }}</span>
      </div>
    </div>

    <!-- 功能入口 -->
    <div class="actions-card">
      <h2 class="card-title">功能入口</h2>

      <div class="action-buttons">
        <button class="action-btn" @click="goToList">
          <span class="action-icon">📋</span>
          <span class="action-text">获取今日推荐列表</span>
          <span class="action-desc">查看当日已缓存的推荐结果</span>
        </button>

        <button class="action-btn" @click="goToPreview">
          <span class="action-icon">🔍</span>
          <span class="action-text">预览推荐</span>
          <span class="action-desc">查看全部题目的评分和入选状态</span>
        </button>

        <button class="action-btn danger" @click="openRegenModal" :disabled="regenerating">
          <span class="action-icon">🔄</span>
          <span class="action-text">{{ regenerating ? '重新生成中...' : '重新生成今日推荐' }}</span>
          <span class="action-desc">删除缓存并重新生成今日推荐（可撤回）</span>
        </button>
      </div>

      <div v-if="regenResult" class="regen-result" :class="{ error: regenResult.includes('失败') }">
        {{ regenResult }}
      </div>
    </div>

    <!-- 确认弹窗 -->
    <div v-if="showRegenModal" class="modal-overlay" @click.self="cancelRegen">
      <div class="modal-box">
        <h3 class="modal-title">确认重新生成</h3>
        <p class="modal-body">
          此操作将删除今日推荐缓存并重新生成推荐列表。<br />
          重新生成后，今日推荐列表会发生变化。<br /><br />
          此操作会记录到操作日志中。
        </p>
        <div class="modal-actions">
          <button class="modal-btn confirm" @click="confirmRegen">确认重新生成</button>
          <button class="modal-btn cancel" @click="cancelRegen">取消</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dev-center {
  max-width: 700px;
  width: 100%;
  padding-bottom: 40px;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
}

.page-title {
  font-size: 28px;
  color: #333;
}

.back-btn {
  padding: 8px 20px;
  background-color: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 6px;
  color: #666;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.15s;
}

.back-btn:hover {
  background-color: #e8e8e8;
}

.stats-card,
.actions-card {
  background-color: #fff;
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

.loading-text {
  color: #999;
  text-align: center;
  padding: 20px;
}

.error-text {
  color: #e74c3c;
  text-align: center;
  padding: 20px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px;
  background-color: #f9fafb;
  border-radius: 8px;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: #4CAF50;
}

.stat-label {
  font-size: 13px;
  color: #888;
  margin-top: 4px;
}

.archived-list {
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px solid #f0f0f0;
  font-size: 13px;
  color: #666;
}

.archived-label {
  color: #999;
}

.archived-names {
  color: #666;
}

.action-buttons {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background-color: #f9fafb;
  border: 1px solid #e8e8e8;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
  text-align: left;
}

.action-btn:hover {
  background-color: #f0f4f0;
  border-color: #4CAF50;
}

.action-btn.danger:hover {
  background-color: #fff5f5;
  border-color: #e74c3c;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.action-icon {
  font-size: 20px;
  flex-shrink: 0;
}

.action-text {
  font-size: 15px;
  font-weight: 500;
  color: #333;
}

.action-desc {
  font-size: 12px;
  color: #999;
  margin-left: auto;
}

.regen-result {
  margin-top: 12px;
  padding: 10px 16px;
  background-color: #e8f5e9;
  border-radius: 6px;
  font-size: 14px;
  color: #2e7d32;
}

.regen-result.error {
  background-color: #ffebee;
  color: #c62828;
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-box {
  background-color: #fff;
  border-radius: 12px;
  padding: 32px;
  max-width: 420px;
  width: 90%;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.15);
}

.modal-title {
  font-size: 18px;
  font-weight: 600;
  color: #333;
  margin-bottom: 16px;
}

.modal-body {
  font-size: 14px;
  color: #666;
  line-height: 1.6;
  margin-bottom: 24px;
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.modal-btn {
  padding: 10px 24px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: all 0.15s;
}

.modal-btn.confirm {
  background-color: #e74c3c;
  color: #fff;
}

.modal-btn.confirm:hover {
  background-color: #c0392b;
}

.modal-btn.cancel {
  background-color: #f5f5f5;
  color: #666;
  border: 1px solid #ddd;
}

.modal-btn.cancel:hover {
  background-color: #e8e8e8;
}
</style>
