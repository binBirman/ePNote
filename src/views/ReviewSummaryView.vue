<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getTodayReviewRecords, getDailyReviewStatus } from '@/api/review'
import type { ReviewResult } from '@/types/question'

// 复习结果项
interface ReviewResultItem {
  questionId: number
  questionName: string
  result: ReviewResult
}

const router = useRouter()
const route = useRoute()

const reviewResults = ref<ReviewResultItem[]>([])
const loading = ref(false)

// 从后端获取今日复习记录
onMounted(async () => {
  loading.value = true

  try {
    // 获取今日复习状态
    const status = await getDailyReviewStatus()

    if (status.reviewed_count === 0) {
      // 没有复习记录，跳转回复习页
      router.push('/review')
      return
    }

    // 获取复习记录详情
    const records = await getTodayReviewRecords()

    reviewResults.value = records.map(r => ({
      questionId: r.question_id,
      questionName: r.question_name || '未命名题目',
      result: r.result as ReviewResult
    }))
  } catch (e) {
    console.error('获取复习记录失败:', e)
    router.push('/review')
  } finally {
    loading.value = false
  }
})

// 统计计算
const stats = computed(() => {
  const correct = reviewResults.value.filter(r => r.result === 'CORRECT').length
  const fuzzy = reviewResults.value.filter(r => r.result === 'FUZZY').length
  const wrong = reviewResults.value.filter(r => r.result === 'WRONG').length
  const total = reviewResults.value.length

  return {
    correct,
    fuzzy,
    wrong,
    total,
    accuracy: total > 0 ? Math.round((correct / total) * 100) : 0
  }
})

// 按结果分组
const groupedByResult = computed(() => {
  return {
    wrong: reviewResults.value.filter(r => r.result === 'WRONG'),
    fuzzy: reviewResults.value.filter(r => r.result === 'FUZZY'),
    correct: reviewResults.value.filter(r => r.result === 'CORRECT')
  }
})

// 进入题目详情页
const goToDetail = (questionId: number) => {
  router.push(`/questions/${questionId}`)
}

// 返回复习页
const goBack = () => {
  router.push('/review')
}

// 重新复习错题
const reviewWrong = () => {
  // 提取错题ID
  const wrongIds = groupedByResult.value.wrong.map(r => r.questionId)
  if (wrongIds.length > 0) {
    router.push({
      path: '/review/session',
      query: {
        subject: 'ALL',
        limit: wrongIds.length.toString(),
        reviewWrong: 'true'
      }
    })
  }
}

// 获取结果对应的样式
const getResultClass = (result: ReviewResult) => {
  return {
    'result-tag': true,
    'correct': result === 'CORRECT',
    'fuzzy': result === 'FUZZY',
    'wrong': result === 'WRONG'
  }
}

const getResultText = (result: ReviewResult) => {
  switch (result) {
    case 'CORRECT': return '记得'
    case 'FUZZY': return '模糊'
    case 'WRONG': return '不记得'
    default: return result
  }
}
</script>

<template>
  <div class="summary-container">
    <!-- 加载状态 -->
    <div v-if="loading" class="loading">
      <div class="loading-text">加载中...</div>
    </div>

    <template v-else>
      <h1 class="page-title">复习总结</h1>
      <div class="summary-actions">
        <button class="back-btn" @click="goBack">返回复习首页</button>
      </div>

      <!-- 统计卡片 -->
      <div class="stats-cards">
        <div class="stat-card total">
          <div class="stat-value">{{ stats.total }}</div>
          <div class="stat-label">总题数</div>
        </div>
        <div class="stat-card correct">
          <div class="stat-value">{{ stats.correct }}</div>
          <div class="stat-label">记得</div>
        </div>
        <div class="stat-card fuzzy">
          <div class="stat-value">{{ stats.fuzzy }}</div>
          <div class="stat-label">模糊</div>
        </div>
        <div class="stat-card wrong">
          <div class="stat-value">{{ stats.wrong }}</div>
          <div class="stat-label">不记得</div>
        </div>
        <div class="stat-card accuracy">
          <div class="stat-value">{{ stats.accuracy }}%</div>
          <div class="stat-label">正确率</div>
        </div>
      </div>

      <!-- 重新复习错题按钮 -->
      <div v-if="stats.wrong > 0" class="review-wrong-section">
        <button class="review-wrong-btn" @click="reviewWrong">
          重新复习错题 ({{ stats.wrong }} 题)
        </button>
      </div>

      <!-- 按结果分组展示 -->
      <div class="result-sections">
        <!-- 错题 -->
        <div v-if="groupedByResult.wrong.length > 0" class="result-section">
          <h2 class="section-title wrong-title">
            不记得 ({{ groupedByResult.wrong.length }})
          </h2>
          <div class="question-list">
            <div
              v-for="item in groupedByResult.wrong"
              :key="item.questionId"
              class="question-item"
              @click="goToDetail(item.questionId)"
            >
              <span class="question-name">{{ item.questionName || '未命名题目' }}</span>
              <span class="result-tag wrong">不记得</span>
            </div>
          </div>
        </div>

        <!-- 模糊题 -->
        <div v-if="groupedByResult.fuzzy.length > 0" class="result-section">
          <h2 class="section-title fuzzy-title">
            模糊 ({{ groupedByResult.fuzzy.length }})
          </h2>
          <div class="question-list">
            <div
              v-for="item in groupedByResult.fuzzy"
              :key="item.questionId"
              class="question-item"
              @click="goToDetail(item.questionId)"
            >
              <span class="question-name">{{ item.questionName || '未命名题目' }}</span>
              <span class="result-tag fuzzy">模糊</span>
            </div>
          </div>
        </div>

        <!-- 正确题 -->
        <div v-if="groupedByResult.correct.length > 0" class="result-section">
          <h2 class="section-title correct-title">
            记得 ({{ groupedByResult.correct.length }})
          </h2>
          <div class="question-list">
            <div
              v-for="item in groupedByResult.correct"
              :key="item.questionId"
              class="question-item"
              @click="goToDetail(item.questionId)"
            >
              <span class="question-name">{{ item.questionName || '未命名题目' }}</span>
              <span class="result-tag correct">记得</span>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.summary-container {
  max-width: 800px;
  width: 100%;
  margin: 0 auto;
  padding: 24px;
}

.loading {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
}

.loading-text {
  font-size: 16px;
  color: #666;
}

.summary-actions {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 24px;
}

.page-title {
  font-size: 32px;
  color: #333;
  margin-bottom: 30px;
  text-align: center;
}

.back-btn {
  padding: 8px 16px;
  background-color: #fff;
  border: 2px solid #4CAF50;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  color: #4CAF50;
  font-weight: 500;
  transition: all 0.2s;
}

.back-btn:hover {
  background-color: #4CAF50;
  color: #fff;
}

/* 统计卡片 */
.stats-cards {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  background: #fff;
  border-radius: 12px;
  padding: 20px;
  text-align: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.stat-card.total {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}

.stat-card.correct {
  background: linear-gradient(135deg, #4CAF50 0%, #2E7D32 100%);
  color: #fff;
}

.stat-card.fuzzy {
  background: linear-gradient(135deg, #FF9800 0%, #F57C00 100%);
  color: #fff;
}

.stat-card.wrong {
  background: linear-gradient(135deg, #f44336 0%, #d32f2f 100%);
  color: #fff;
}

.stat-card.accuracy {
  background: linear-gradient(135deg, #2196F3 0%, #1565C0 100%);
  color: #fff;
}

.stat-value {
  font-size: 32px;
  font-weight: 700;
}

.stat-label {
  font-size: 14px;
  opacity: 0.9;
  margin-top: 4px;
}

/* 重新复习错题 */
.review-wrong-section {
  margin-bottom: 24px;
  text-align: center;
}

.review-wrong-btn {
  padding: 12px 24px;
  background-color: #fff;
  color: #f44336;
  border: 2px solid #f44336;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.review-wrong-btn:hover {
  background-color: #f44336;
  color: #fff;
}

/* 结果分组 */
.result-sections {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.result-section {
  background: #fff;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 16px 0;
  padding-bottom: 12px;
  border-bottom: 2px solid #f0f0f0;
}

.wrong-title {
  color: #f44336;
  border-color: #f44336;
}

.fuzzy-title {
  color: #ff9800;
  border-color: #ff9800;
}

.correct-title {
  color: #4CAF50;
  border-color: #4CAF50;
}

/* 题目列表 */
.question-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.question-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f9f9f9;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.question-item:hover {
  background: #f0f0f0;
  transform: translateX(4px);
}

.question-name {
  font-size: 14px;
  color: #333;
}

.result-tag {
  padding: 4px 12px;
  border-radius: 16px;
  font-size: 12px;
  font-weight: 500;
}

.result-tag.correct {
  background: #e8f5e9;
  color: #2E7D32;
}

.result-tag.fuzzy {
  background: #fff3e0;
  color: #F57C00;
}

.result-tag.wrong {
  background: #ffebee;
  color: #d32f2f;
}

/* 响应式 */
@media (max-width: 600px) {
  .stats-cards {
    grid-template-columns: repeat(3, 1fr);
  }

  .stat-card.accuracy {
    grid-column: span 3;
  }
}
</style>
