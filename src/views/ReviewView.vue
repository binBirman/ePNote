<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getSubjects, filterQuestionsByState, type Question } from '../mock/data'

const router = useRouter()

const subjects = ref<string[]>([])
const selectedSubject = ref<string>('ALL')
const pendingQuestions = ref<Question[]>([])

onMounted(() => {
  subjects.value = ['全部', ...getSubjects()]
  selectedSubject.value = '全部'
  loadPendingQuestions()
})

const loadPendingQuestions = () => {
  // 获取需要复习的题目（NEW 和 LEARNING 状态）
  pendingQuestions.value = filterQuestionsByState('ALL').filter(
    q => q.state === 'NEW' || q.state === 'LEARNING'
  )
}

const pendingCount = computed(() => {
  if (selectedSubject.value === '全部') {
    return pendingQuestions.value.length
  }
  return pendingQuestions.value.filter(q => q.subject === selectedSubject.value).length
})

const startReview = () => {
  router.push({
    name: 'review-session',
    query: { subject: selectedSubject.value === '全部' ? 'ALL' : selectedSubject.value }
  })
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
          @click="selectedSubject = subject"
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

    <button class="start-btn" @click="startReview">
      开始复习
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
  max-width: 600px;
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
