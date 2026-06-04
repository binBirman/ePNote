<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { previewRecommendation } from '@/api/review'
import type { PreviewRecommendationItem } from '@/api/review'

const router = useRouter()

const items = ref<PreviewRecommendationItem[]>([])
const loading = ref(true)
const error = ref('')
const showScoreDetail = ref(false)
const showExclusionReason = ref(true)

const selectedCount = computed(() => items.value.filter(i => i.selected).length)

onMounted(async () => {
  await loadPreview()
})

async function loadPreview() {
  loading.value = true
  error.value = ''
  try {
    items.value = await previewRecommendation(
      showScoreDetail.value,
      showExclusionReason.value,
    )
  } catch (e) {
    error.value = '加载预览失败: ' + String(e)
  } finally {
    loading.value = false
  }
}

// 切换复选框时重新加载数据
async function onShowScoreDetailChange() {
  await loadPreview()
}

async function onShowExclusionReasonChange() {
  await loadPreview()
}

function goBack() {
  router.push('/dev/center')
}
</script>

<template>
  <div class="preview-page">
    <div class="page-header">
      <h1 class="page-title">预览推荐</h1>
      <button class="back-btn" @click="goBack">返回开发者中心</button>
    </div>

    <!-- 复选框控制 -->
    <div class="controls-card">
      <label class="checkbox-label">
        <input
          type="checkbox"
          v-model="showScoreDetail"
          @change="onShowScoreDetailChange"
        />
        <span>显示评分详情</span>
      </label>
      <label class="checkbox-label">
        <input
          type="checkbox"
          v-model="showExclusionReason"
          @change="onShowExclusionReasonChange"
        />
        <span>显示落选原因</span>
      </label>
    </div>

    <!-- 错误/加载 -->
    <div v-if="loading" class="loading-text">加载中...</div>
    <div v-else-if="error" class="error-text">{{ error }}</div>

    <template v-else>
      <!-- 汇总 -->
      <div class="summary-bar">
        共 <strong>{{ items.length }}</strong> 题，入选 <strong class="selected-num">{{ selectedCount }}</strong> 题
      </div>

      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>题目名</th>
              <th>科目</th>
              <th>分数</th>
              <th>状态</th>
              <th>推荐理由</th>
              <th>排名 / 上限</th>
              <th v-if="showExclusionReason">落选原因</th>
              <th v-if="showScoreDetail">遗忘风险</th>
              <th v-if="showScoreDetail">新鲜度</th>
              <th v-if="showScoreDetail">错误加成</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in items"
              :key="item.question_id"
              :class="item.selected ? 'row-selected' : 'row-excluded'"
            >
              <td class="name-cell">{{ item.name || '（无标题）' }}</td>
              <td>{{ item.subject || '-' }}</td>
              <td class="score-cell">{{ item.score.toFixed(2) }}</td>
              <td>
                <span v-if="item.selected" class="badge selected">入选</span>
                <span v-else class="badge excluded">落选</span>
              </td>
              <td class="reason-cell">
                <span v-if="item.reason.length > 0">{{ item.reason.join('、') }}</span>
                <span v-else class="no-reason">-</span>
              </td>
              <td class="rank-cell">{{ item.subject_rank }} / {{ item.subject_limit || '-' }}</td>
              <td v-if="showExclusionReason" class="reason-cell">
                <span v-if="item.exclusion_reason.length > 0">{{ item.exclusion_reason.join('、') }}</span>
                <span v-else class="no-reason">-</span>
              </td>
              <td v-if="showScoreDetail" class="num-cell">
                {{ item.score_detail ? item.score_detail.forget_risk.toFixed(2) : '-' }}
              </td>
              <td v-if="showScoreDetail" class="num-cell">
                {{ item.score_detail ? item.score_detail.freshness_bonus.toFixed(2) : '-' }}
              </td>
              <td v-if="showScoreDetail" class="num-cell">
                {{ item.score_detail ? item.score_detail.error_rate_bonus.toFixed(2) : '-' }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </div>
</template>

<style scoped>
.preview-page {
  max-width: 100%;
  width: 100%;
  padding-bottom: 40px;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
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

.controls-card {
  display: flex;
  gap: 24px;
  padding: 12px 20px;
  background-color: #fff;
  border-radius: 8px;
  margin-bottom: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: #333;
  cursor: pointer;
}

.checkbox-label input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: #4CAF50;
  cursor: pointer;
}

.loading-text {
  color: #999;
  text-align: center;
  padding: 40px;
}

.error-text {
  color: #e74c3c;
  text-align: center;
  padding: 40px;
}

.summary-bar {
  margin-bottom: 12px;
  font-size: 14px;
  color: #666;
}

.selected-num {
  color: #4CAF50;
}

.table-wrap {
  overflow-x: auto;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  background-color: #fff;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  font-size: 13px;
}

.data-table th {
  background-color: #f9fafb;
  padding: 10px 12px;
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: #666;
  border-bottom: 2px solid #eee;
  white-space: nowrap;
}

.data-table td {
  padding: 8px 12px;
  border-bottom: 1px solid #f5f5f5;
  color: #333;
}

.row-selected {
  background-color: #e8f5e9;
}

.row-selected:hover {
  background-color: #c8e6c9;
}

.row-excluded {
  background-color: #fff5f5;
}

.row-excluded:hover {
  background-color: #ffebee;
}

.name-cell {
  font-weight: 500;
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.score-cell {
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.badge {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 500;
}

.badge.selected {
  background-color: #4CAF50;
  color: #fff;
}

.badge.excluded {
  background-color: #e74c3c;
  color: #fff;
}

.reason-cell {
  color: #888;
  font-size: 12px;
  max-width: 200px;
}

.rank-cell {
  font-variant-numeric: tabular-nums;
  text-align: center;
}

.num-cell {
  font-variant-numeric: tabular-nums;
  text-align: center;
  color: #666;
}

.no-reason {
  color: #ccc;
}
</style>
