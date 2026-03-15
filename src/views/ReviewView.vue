<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getRecommendationList, getDailyReviewStatus, listSubjects, getDailyRecommendation } from '@/api/review'

const router = useRouter()

const subjects = ref<string[]>([])
const selectedSubject = ref<string>('ALL')
const pendingQuestions = ref<any[]>([])
const loading = ref(false)
const reviewLimit = ref<number>(10) // 默认复习10题
const maxLimit = ref<number>(50) // 最大限制

// 每日复习状态
const reviewStatus = ref({
  recommended_count: 0,
  reviewed_count: 0,
  is_completed: false
})

onMounted(async () => {
  loading.value = true
  try {
    // 先调用 getDailyRecommendation 生成当日推荐（如有必要）
    await getDailyRecommendation(50)

    // 获取科目列表
    const subjectList = await listSubjects()
    subjects.value = ['全部', ...subjectList]
    selectedSubject.value = '全部'

    // 获取每日复习状态
    await loadReviewStatus()

    // 加载推荐题目
    await loadPendingQuestions()
  } catch (e) {
    console.error('加载数据失败:', e)
  } finally {
    loading.value = false
  }
})

// 加载每日复习状态
const loadReviewStatus = async () => {
  try {
    const status = await getDailyReviewStatus()
    reviewStatus.value = status
  } catch (e) {
    console.error('获取复习状态失败:', e)
  }
}

const loadPendingQuestions = async () => {
  try {
    // 使用新推荐系统获取每日推荐
    const result = await getRecommendationList(maxLimit.value)
    pendingQuestions.value = result
  } catch (e) {
    console.error('加载推荐题目失败:', e)
    pendingQuestions.value = []
  }
}

// 当科目选择变化时重新加载
const onSubjectChange = async (subject: string) => {
  selectedSubject.value = subject
  await loadPendingQuestions()
}

const pendingCount = computed(() => {
  return pendingQuestions.value.length
})

// 是否已完成今日推荐
const isCompleted = computed(() => reviewStatus.value.is_completed)

// 按钮文本
const buttonText = computed(() => {
  if (reviewStatus.value.recommended_count === 0) {
    return '开始复习'
  }
  return isCompleted.value ? '查看结果' : '开始复习'
})

// 计算实际可复习的题数（不超过待复习总数）
const actualLimit = computed(() => {
  return Math.min(reviewLimit.value, pendingCount.value)
})

const startReview = () => {
  if (isCompleted.value) {
    // 已完成，跳转到总结页面
    router.push('/review/summary')
  } else {
    // 开始复习
    router.push({
      name: 'review-session',
      query: {
        subject: selectedSubject.value === '全部' ? 'ALL' : selectedSubject.value,
        limit: reviewLimit.value.toString()
      }
    })
  }
}
</script>

<template>
  <div class="review-container">
    <h1 class="page-title">复习</h1>

    <!-- 科目选择 -->
    <div class="subject-selector">
      <div class="selector-label">选择科目：</div>
      <div class="subject-tags">
        <button
          v-for="subject in subjects"
          :key="subject"
          class="subject-tag"
          :class="{ active: selectedSubject === subject }"
          @click="onSubjectChange(subject)"
        >
          {{ subject }}
        </button>
      </div>
    </div>

    <div class="stats-card">
      <div class="today-section">
        <div class="stat-label">{{ selectedSubject === '全部' ? '今日待复习' : selectedSubject + '待复习' }}</div>
        <div class="stat-number">{{ pendingCount }}</div>
        <div class="stat-unit">题</div>
      </div>
    </div>

    <!-- 复习题数限制 -->
    <div class="limit-selector">
      <div class="selector-label">本次复习题数：</div>
      <div class="limit-input-group">
        <button class="limit-btn" @click="reviewLimit = Math.max(1, reviewLimit - 5)">-5</button>
        <button class="limit-btn" @click="reviewLimit = Math.max(1, reviewLimit - 1)">-</button>
        <input
          v-model.number="reviewLimit"
          type="number"
          class="limit-input"
          min="1"
          :max="pendingCount"
          :disabled="isCompleted"
        />
        <button class="limit-btn" @click="reviewLimit = Math.min(pendingCount, reviewLimit + 1)" :disabled="isCompleted">+</button>
        <button class="limit-btn" @click="reviewLimit = Math.min(pendingCount, reviewLimit + 5)" :disabled="isCompleted">+5</button>
      </div>
      <div class="limit-hint">（共 {{ pendingCount }} 题可复习）</div>
    </div>

    <!-- 复习状态 -->
    <div v-if="reviewStatus.recommended_count > 0" class="status-card">
      <div class="status-info">
        <span class="status-label">今日进度：</span>
        <span class="status-value">{{ reviewStatus.reviewed_count }} / {{ reviewStatus.recommended_count }}</span>
        <span v-if="isCompleted" class="status-completed">（已完成）</span>
        <span v-else class="status-remaining">（还剩 {{ reviewStatus.recommended_count - reviewStatus.reviewed_count }} 题）</span>
      </div>
      <div class="status-progress">
        <div
          class="status-progress-bar"
          :style="{ width: (reviewStatus.reviewed_count / reviewStatus.recommended_count * 100) + '%' }"
        ></div>
      </div>
    </div>

    <button
      class="start-btn"
      :class="{ completed: isCompleted }"
      :disabled="actualLimit === 0 && !isCompleted"
      @click="startReview"
    >
      {{ buttonText }} {{ !isCompleted && actualLimit > 0 ? `（${actualLimit}题）` : '' }}
    </button>

    <div class="info-card">
      <h3 class="info-title">复习建议</h3>
      <ul class="info-list">
        <li>{{ selectedSubject === '全部' ? '总共有' : selectedSubject + '有' }} {{ pendingCount }} 道题目需要复习</li>
        <li>建议每日复习时间控制在 30-45 分钟</li>
        <li>遇到不确定的题目选择"模糊"选项</li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.review-container {
  max-width: 800px;
  width: 100%;
}

.page-title {
  font-size: 32px;
  color: #333;
  margin-bottom: 30px;
  text-align: center;
}

.subject-selector {
  margin-bottom: 24px;
  background-color: #ffffff;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.selector-label {
  color: #333;
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 16px;
}

.subject-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.subject-tag {
  padding: 10px 20px;
  background-color: #fafafa;
  border: 1px solid #ddd;
  border-radius: 8px;
  color: #333;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.subject-tag:hover {
  background-color: #f0f0f0;
  border-color: #bbb;
}

.subject-tag.active {
  background-color: #4CAF50;
  border-color: #4CAF50;
  color: #ffffff;
}

.stats-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 32px;
  display: flex;
  align-items: center;
  justify-content: space-around;
  margin-bottom: 24px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.today-section,
.yesterday-section {
  text-align: center;
}

.stat-label {
  color: #666;
  font-size: 14px;
  margin-bottom: 8px;
}

.stat-number {
  font-size: 56px;
  font-weight: bold;
  color: #4CAF50;
  line-height: 1;
}

.stat-number.small {
  font-size: 36px;
  color: #333;
}

.stat-unit {
  color: #666;
  font-size: 14px;
  margin-top: 4px;
}

.divider {
  width: 1px;
  height: 80px;
  background-color: #e0e0e0;
}

/* 复习状态卡片 */
.status-card {
  background: #fff;
  border-radius: 12px;
  padding: 16px 20px;
  margin-bottom: 20px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.status-info {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
}

.status-label {
  font-size: 14px;
  color: #666;
}

.status-value {
  font-size: 18px;
  font-weight: 600;
  color: #333;
  margin-right: 8px;
}

.status-completed {
  font-size: 14px;
  color: #4CAF50;
  font-weight: 500;
}

.status-remaining {
  font-size: 14px;
  color: #ff9800;
}

.status-progress {
  height: 8px;
  background-color: #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
}

.status-progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #4CAF50, #8BC34A);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.start-btn {
  width: 100%;
  padding: 18px;
  background-color: #4CAF50;
  border: none;
  border-radius: 12px;
  color: #ffffff;
  font-size: 18px;
  font-weight: 600;
  transition: all 0.2s;
  margin-bottom: 24px;
}

.start-btn:hover {
  background-color: #45a049;
  transform: translateY(-2px);
}

.start-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
  transform: none;
}

.start-btn.completed {
  background: linear-gradient(135deg, #2196F3, #1976D2);
}

.start-btn.completed:hover {
  background: linear-gradient(135deg, #1E88E5, #1565C0);
}

.limit-selector {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 24px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.selector-label {
  color: #333;
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 16px;
}

.limit-input-group {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.limit-btn {
  padding: 8px 16px;
  background-color: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 6px;
  color: #333;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.limit-btn:hover {
  background-color: #e0e0e0;
  border-color: #ccc;
}

.limit-input {
  width: 80px;
  padding: 10px;
  text-align: center;
  font-size: 18px;
  font-weight: 600;
  border: 2px solid #4CAF50;
  border-radius: 8px;
  color: #333;
}

.limit-input:focus {
  outline: none;
  border-color: #45a049;
}

.limit-hint {
  text-align: center;
  color: #999;
  font-size: 13px;
  margin-top: 12px;
}

.info-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.info-title {
  font-size: 16px;
  color: #333;
  margin-bottom: 16px;
  font-weight: 600;
}

.info-list {
  list-style: none;
}

.info-list li {
  color: #666;
  padding: 8px 0;
  padding-left: 20px;
  position: relative;
}

.info-list li::before {
  content: '•';
  position: absolute;
  left: 0;
  color: #4CAF50;
}
</style>
