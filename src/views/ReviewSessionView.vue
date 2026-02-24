<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getQuestions, filterQuestionsBySubject, type Question } from '../mock/data'

type ReviewResult = 'WRONG' | 'FUZZY' | 'CORRECT'

const router = useRouter()
const route = useRoute()

const questions = ref<Question[]>([])
const currentIndex = ref(0)
const showAnswer = ref(false)
const reviewResults = ref<{ questionId: number; result: ReviewResult }[]>([])
const isComplete = ref(false)
const selectedSubject = ref<string>('ALL')

onMounted(() => {
  selectedSubject.value = (route.query.subject as string) || 'ALL'

  // 获取待复习的题目（NEW 和 LEARNING 状态）
  let allPendingQuestions = getQuestions().filter(
    q => q.state === 'NEW' || q.state === 'LEARNING'
  )

  // 按科目筛选
  if (selectedSubject.value !== 'ALL') {
    allPendingQuestions = allPendingQuestions.filter(q => q.subject === selectedSubject.value)
  }

  questions.value = allPendingQuestions

  // 如果没有题目，返回复习页
  if (questions.value.length === 0) {
    router.push('/review')
  }
})

const currentQuestion = computed(() => questions.value[currentIndex.value])
const progress = computed(() => `${currentIndex.value + 1} / ${questions.value.length}`)

const toggleAnswer = () => {
  showAnswer.value = !showAnswer.value
}

const handleReview = (result: ReviewResult) => {
  const q = currentQuestion.value
  if (!q) return

  reviewResults.value.push({
    questionId: q.id,
    result
  })

  showAnswer.value = false
  currentIndex.value++

  if (currentIndex.value >= questions.value.length) {
    isComplete.value = true
  }
}

const goBack = () => {
  router.push('/review')
}
</script>

<template>
  <div class="session-container">
    <div v-if="!isComplete && currentQuestion" class="review-session">
      <!-- 顶部进度 -->
      <div class="progress-bar">
        <div class="progress-info">{{ progress }}</div>
        <div class="progress-line">
          <div
            class="progress-fill"
            :style="{ width: ((currentIndex + 1) / questions.length * 100) + '%' }"
          ></div>
        </div>
      </div>

      <!-- 题目区域 -->
      <div class="question-card">
        <!-- 题目信息 -->
        <div class="question-header">
          <div class="question-name">{{ currentQuestion.name }}</div>
          <div class="question-meta">
            <span class="meta-tag">{{ currentQuestion.subject }}</span>
            <span class="meta-tag">{{ currentQuestion.knowledgePoint }}</span>
          </div>
        </div>

        <!-- 题目图区域 -->
        <div class="question-image-placeholder">
          <div class="placeholder-text">题目图片：{{ currentQuestion.questionImage }}</div>
        </div>

        <!-- 显示答案按钮 -->
        <button class="show-answer-btn" @click="toggleAnswer">
          {{ showAnswer ? '隐藏答案' : '显示答案' }}
        </button>

        <!-- 答案区域（图片） -->
        <div v-if="showAnswer" class="answer-section">
          <div class="answer-label">答案：</div>
          <div class="answer-image-placeholder">
            <div class="placeholder-text">答案图片：{{ currentQuestion.answerImage }}</div>
          </div>
        </div>
      </div>

      <!-- 底部操作按钮 -->
      <div v-if="showAnswer" class="action-buttons">
        <button class="action-btn wrong" @click="handleReview('WRONG')">
          不记得
        </button>
        <button class="action-btn fuzzy" @click="handleReview('FUZZY')">
          模糊
        </button>
        <button class="action-btn correct" @click="handleReview('CORRECT')">
          记得
        </button>
      </div>
    </div>

    <!-- 完成页面 -->
    <div v-else class="complete-view">
      <div class="complete-card">
        <div class="complete-icon">✓</div>
        <h2 class="complete-title">今日复习完成</h2>
        <p class="complete-info">共复习 {{ questions.length }} 题</p>

        <div class="result-summary">
          <div class="result-item">
            <span class="result-count">{{ reviewResults.filter(r => r.result === 'CORRECT').length }}</span>
            <span class="result-label">记得</span>
          </div>
          <div class="result-item">
            <span class="result-count">{{ reviewResults.filter(r => r.result === 'FUZZY').length }}</span>
            <span class="result-label">模糊</span>
          </div>
          <div class="result-item">
            <span class="result-count">{{ reviewResults.filter(r => r.result === 'WRONG').length }}</span>
            <span class="result-label">不记得</span>
          </div>
        </div>

        <button class="back-btn" @click="goBack">
          返回复习首页
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.session-container {
  max-width: 700px;
}

.review-session {
  display: flex;
  flex-direction: column;
  min-height: calc(100vh - 60px);
}

.progress-bar {
  margin-bottom: 24px;
}

.progress-info {
  text-align: center;
  color: #666;
  font-size: 14px;
  margin-bottom: 8px;
}

.progress-line {
  height: 6px;
  background-color: #e0e0e0;
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background-color: #4CAF50;
  transition: width 0.3s ease;
}

.question-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 32px;
  flex: 1;
  display: flex;
  flex-direction: column;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.question-header {
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid #e0e0e0;
}

.question-name {
  font-size: 24px;
  font-weight: 600;
  color: #333;
  margin-bottom: 12px;
}

.question-meta {
  display: flex;
  gap: 12px;
}

.meta-tag {
  padding: 6px 12px;
  background-color: #f0f0f0;
  border-radius: 4px;
  font-size: 13px;
  color: #666;
}

.question-image-placeholder {
  background-color: #fafafa;
  border: 2px dashed #ddd;
  border-radius: 8px;
  height: 300px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 24px;
}

.placeholder-text {
  color: #999;
  font-size: 14px;
}

.show-answer-btn {
  align-self: center;
  padding: 12px 32px;
  background-color: #fff;
  border: 1px solid #4CAF50;
  border-radius: 8px;
  color: #4CAF50;
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 24px;
  transition: all 0.2s;
}

.show-answer-btn:hover {
  background-color: #4CAF50;
  color: #ffffff;
}

.answer-section {
  background-color: #fafafa;
  border-radius: 8px;
  padding: 20px;
  border-left: 4px solid #4CAF50;
}

.answer-label {
  color: #4CAF50;
  font-weight: 600;
  margin-bottom: 12px;
}

.answer-image-placeholder {
  background-color: #fff;
  border: 2px dashed #ddd;
  border-radius: 8px;
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-buttons {
  display: flex;
  gap: 12px;
  margin-top: 24px;
  padding-bottom: 60px;
}

.action-btn {
  flex: 1;
  padding: 14px;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  transition: all 0.2s;
  color: #ffffff;
}

.action-btn.wrong {
  background-color: #f44336;
}

.action-btn.wrong:hover {
  background-color: #d32f2f;
}

.action-btn.fuzzy {
  background-color: #ff9800;
}

.action-btn.fuzzy:hover {
  background-color: #f57c00;
}

.action-btn.correct {
  background-color: #4CAF50;
}

.action-btn.correct:hover {
  background-color: #45a049;
}

.complete-view {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: calc(100vh - 60px);
}

.complete-card {
  background-color: #ffffff;
  border-radius: 16px;
  padding: 48px;
  text-align: center;
  max-width: 400px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.complete-icon {
  font-size: 72px;
  color: #4CAF50;
  margin-bottom: 16px;
}

.complete-title {
  font-size: 28px;
  color: #333;
  margin-bottom: 8px;
}

.complete-info {
  color: #666;
  font-size: 16px;
  margin-bottom: 32px;
}

.result-summary {
  display: flex;
  justify-content: space-around;
  margin-bottom: 32px;
  padding: 24px;
  background-color: #fafafa;
  border-radius: 12px;
}

.result-item {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.result-count {
  font-size: 32px;
  font-weight: bold;
  color: #333;
  margin-bottom: 8px;
}

.result-label {
  font-size: 14px;
  color: #666;
}

.back-btn {
  width: 100%;
  padding: 16px;
  background-color: #4CAF50;
  border: none;
  border-radius: 8px;
  color: #ffffff;
  font-size: 16px;
  font-weight: 600;
  transition: all 0.2s;
}

.back-btn:hover {
  background-color: #45a049;
}
</style>
