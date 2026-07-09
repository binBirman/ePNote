<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getDailyRecommendation } from '@/api/review'
import type { DailyRecommendation, RecommendedQuestion } from '@/api/review'
import { goBack } from '@/utils/back'

const router = useRouter()

const questions = ref<RecommendedQuestion[]>([])
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    const result: DailyRecommendation = await getDailyRecommendation()
    questions.value = result.questions
  } catch (e) {
    error.value = '加载今日推荐失败: ' + String(e)
  } finally {
    loading.value = false
  }
})

function goBackView() {
  goBack(router, '/dev/center')
}
</script>

<template>
  <div class="list-page">
    <div class="page-header">
      <h1 class="page-title">今日推荐列表</h1>
      <button class="back-btn" @click="goBackView">← 返回</button>
    </div>

    <div v-if="loading" class="loading-text">加载中...</div>
    <div v-else-if="error" class="error-text">{{ error }}</div>

    <div v-else-if="questions.length === 0" class="empty-text">
      今日暂无推荐数据
    </div>

    <div v-else class="summary-bar">
      共 <strong>{{ questions.length }}</strong> 题
    </div>

    <table v-if="questions.length > 0" class="data-table">
      <thead>
        <tr>
          <th>题目名</th>
          <th>科目</th>
          <th>分数</th>
          <th>推荐原因</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="q in questions" :key="q.question_id">
          <td class="name-cell">{{ q.name || '（无标题）' }}</td>
          <td>{{ q.subject || '-' }}</td>
          <td class="score-cell">{{ q.score.toFixed(2) }}</td>
          <td class="reason-cell">
            <span v-if="q.reason && q.reason.length > 0">
              {{ q.reason.join('、') }}
            </span>
            <span v-else class="no-reason">-</span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.list-page {
  max-width: 900px;
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

.empty-text {
  text-align: center;
  padding: 40px;
  color: #999;
  font-size: 15px;
}

.summary-bar {
  margin-bottom: 12px;
  font-size: 14px;
  color: #666;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  background-color: #fff;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.data-table th {
  background-color: #f9fafb;
  padding: 12px 16px;
  text-align: left;
  font-size: 13px;
  font-weight: 600;
  color: #666;
  border-bottom: 2px solid #eee;
}

.data-table td {
  padding: 10px 16px;
  border-bottom: 1px solid #f5f5f5;
  font-size: 14px;
  color: #333;
}

.name-cell {
  font-weight: 500;
}

.score-cell {
  font-variant-numeric: tabular-nums;
  color: #4CAF50;
  font-weight: 600;
}

.reason-cell {
  font-size: 13px;
  color: #888;
}

.no-reason {
  color: #ccc;
}
</style>
