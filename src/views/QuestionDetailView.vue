<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getQuestionById, moveToRecycleBin, type QuestionState } from '../mock/data'

const router = useRouter()
const route = useRoute()

const question = ref<any>(null)
const isDeleting = ref(false)

onMounted(() => {
  const id = Number(route.params.id)
  question.value = getQuestionById(id)
  if (!question.value) {
    router.push('/questions')
  }
})

const getStateColor = (state: QuestionState) => {
  switch (state) {
    case 'NEW':
      return '#2196F3'
    case 'LEARNING':
      return '#FF9800'
    case 'STABLE':
      return '#4CAF50'
    default:
      return '#999'
  }
}

const getStateLabel = (state: QuestionState) => {
  switch (state) {
    case 'NEW':
      return '新题'
    case 'LEARNING':
      return '学习中'
    case 'STABLE':
      return '已掌握'
    default:
      return '未知'
  }
}

const handleEdit = () => {
  // 编辑功能在后续实现
  alert('编辑功能将在后续版本中实现')
}

const handleDelete = async () => {
  if (!confirm('确定要删除这道题目吗？题目将移至回收站。')) {
    return
  }

  isDeleting.value = true
  moveToRecycleBin(question.value.id)
  isDeleting.value = false
  router.push('/questions')
}

const goBack = () => {
  router.push('/questions')
}
</script>

<template>
  <div class="detail-container">
    <button class="back-link" @click="goBack">
      ← 返回列表
    </button>

    <div v-if="question && !isDeleting" class="detail-card">
      <!-- 题目标题和状态 -->
      <div class="detail-header">
        <h1 class="question-title">{{ question.name }}</h1>
        <span
          class="state-badge large"
          :style="{ backgroundColor: getStateColor(question.state) }"
        >
          {{ getStateLabel(question.state) }}
        </span>
      </div>

      <!-- 题目信息 -->
      <div class="info-row">
        <span class="info-label">题目 ID：</span>
        <span class="info-value">{{ question.id }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">科目：</span>
        <span class="info-value">{{ question.subject }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">知识点：</span>
        <span class="info-value">{{ question.knowledgePoint }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">创建日期：</span>
        <span class="info-value">{{ question.createdDate }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">上次复习：</span>
        <span class="info-value">{{ question.lastReviewed || '从未' }}</span>
      </div>

      <!-- 题目图占位区域 -->
      <div class="image-placeholder">
        <div class="placeholder-text">题目图片：{{ question.questionImage }}</div>
      </div>

      <!-- 答案区域（图片） -->
      <div class="answer-section">
        <h3 class="section-title">答案</h3>
        <div class="answer-image-placeholder">
          <div class="placeholder-text">答案图片：{{ question.answerImage }}</div>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="action-buttons">
        <button class="action-btn edit" @click="handleEdit">
          编辑
        </button>
        <button class="action-btn delete" @click="handleDelete">
          删除
        </button>
      </div>
    </div>

    <div v-else-if="isDeleting" class="loading">
      删除中...
    </div>
  </div>
</template>

<style scoped>
.detail-container {
  max-width: 700px;
}

.back-link {
  background: none;
  border: none;
  color: #4CAF50;
  font-size: 14px;
  cursor: pointer;
  padding: 8px 0;
  margin-bottom: 20px;
  transition: color 0.2s;
}

.back-link:hover {
  color: #45a049;
}

.detail-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 32px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 24px;
  padding-bottom: 24px;
  border-bottom: 1px solid #e0e0e0;
}

.question-title {
  flex: 1;
  font-size: 24px;
  color: #333;
  line-height: 1.4;
  margin: 0;
}

.state-badge {
  padding: 6px 14px;
  border-radius: 14px;
  color: #ffffff;
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
}

.state-badge.large {
  font-size: 14px;
  padding: 8px 16px;
}

.info-row {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  color: #666;
  font-size: 14px;
}

.info-label {
  font-weight: 500;
}

.info-value {
  color: #333;
}

.image-placeholder {
  background-color: #fafafa;
  border: 2px dashed #ddd;
  border-radius: 8px;
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 24px 0;
}

.placeholder-text {
  color: #999;
  font-size: 14px;
}

.answer-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 16px;
  color: #333;
  margin-bottom: 16px;
  font-weight: 600;
}

.answer-image-placeholder {
  background-color: #fff;
  border: 2px dashed #ddd;
  border-radius: 8px;
  height: 250px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-buttons {
  display: flex;
  gap: 12px;
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

.action-btn.edit {
  background-color: #2196F3;
}

.action-btn.edit:hover {
  background-color: #1976D2;
}

.action-btn.delete {
  background-color: #f44336;
}

.action-btn.delete:hover {
  background-color: #d32f2f;
}

.loading {
  text-align: center;
  padding: 60px 20px;
  color: #666;
  font-size: 16px;
}
</style>
