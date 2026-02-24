<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { addQuestion } from '../mock/data'

const router = useRouter()

const questionName = ref('')
const questionSubject = ref('')
const questionKnowledgePoint = ref('')
const questionQuestionImage = ref('')
const questionAnswerImage = ref('')

const handleSubmit = () => {
  if (!questionName.value.trim()) {
    alert('请输入题目名')
    return
  }

  if (!questionSubject.value.trim()) {
    alert('请输入科目')
    return
  }

  if (!questionKnowledgePoint.value.trim()) {
    alert('请输入知识点')
    return
  }

  if (!questionQuestionImage.value.trim()) {
    alert('请选择题目图片')
    return
  }

  if (!questionAnswerImage.value.trim()) {
    alert('请选择答案图片')
    return
  }

  addQuestion({
    name: questionName.value,
    subject: questionSubject.value,
    knowledgePoint: questionKnowledgePoint.value,
    questionImage: questionQuestionImage.value,
    answerImage: questionAnswerImage.value,
    state: 'NEW'
  })

  // 清空表单
  questionName.value = ''
  questionSubject.value = ''
  questionKnowledgePoint.value = ''
  questionQuestionImage.value = ''
  questionAnswerImage.value = ''

  alert('题目创建成功！')
  router.push('/questions')
}

const handleCancel = () => {
  if (questionName.value || questionSubject.value || questionKnowledgePoint.value ||
      questionQuestionImage.value || questionAnswerImage.value) {
    if (!confirm('确定要放弃当前编辑吗？')) {
      return
    }
  }
  router.push('/questions')
}

const selectQuestionImage = () => {
  // Mock 文件选择
  questionQuestionImage.value = `question-${Date.now()}.png`
}

const selectAnswerImage = () => {
  // Mock 文件选择
  questionAnswerImage.value = `answer-${Date.now()}.png`
}
</script>

<template>
  <div class="new-container">
    <div class="back-section">
      <button class="back-link" @click="handleCancel">
        ← 返回列表
      </button>
    </div>

    <div class="new-card">
      <h1 class="page-title">新建题目</h1>

      <form @submit.prevent="handleSubmit" class="question-form">
        <!-- 题目名 -->
        <div class="form-group">
          <label class="form-label" for="name">题目名 *</label>
          <input
            id="name"
            v-model="questionName"
            type="text"
            class="form-input"
            placeholder="请输入题目名..."
            required
          />
        </div>

        <!-- 科目 -->
        <div class="form-group">
          <label class="form-label" for="subject">科目 *</label>
          <input
            id="subject"
            v-model="questionSubject"
            type="text"
            class="form-input"
            placeholder="请输入科目..."
            required
          />
        </div>

        <!-- 知识点 -->
        <div class="form-group">
          <label class="form-label" for="knowledgePoint">知识点 *</label>
          <input
            id="knowledgePoint"
            v-model="questionKnowledgePoint"
            type="text"
            class="form-input"
            placeholder="请输入知识点..."
            required
          />
        </div>

        <!-- 题目图 -->
        <div class="form-group">
          <label class="form-label">题目图 *</label>
          <div class="image-upload-area">
            <div v-if="!questionQuestionImage" class="upload-placeholder" @click="selectQuestionImage">
              <span class="upload-icon">📷</span>
              <span class="upload-text">点击选择题目图片</span>
            </div>
            <div v-else class="upload-preview">
              <span class="preview-text">{{ questionQuestionImage }}</span>
              <button type="button" class="remove-btn" @click="questionQuestionImage = ''">删除</button>
            </div>
          </div>
        </div>

        <!-- 答案图 -->
        <div class="form-group">
          <label class="form-label">答案图 *</label>
          <div class="image-upload-area">
            <div v-if="!questionAnswerImage" class="upload-placeholder" @click="selectAnswerImage">
              <span class="upload-icon">📷</span>
              <span class="upload-text">点击选择答案图片</span>
            </div>
            <div v-else class="upload-preview">
              <span class="preview-text">{{ questionAnswerImage }}</span>
              <button type="button" class="remove-btn" @click="questionAnswerImage = ''">删除</button>
            </div>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="form-actions">
          <button type="button" class="btn cancel" @click="handleCancel">
            取消
          </button>
          <button type="submit" class="btn submit">
            创建题目
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.new-container {
  max-width: 700px;
}

.back-section {
  margin-bottom: 16px;
}

.back-link {
  background: none;
  border: none;
  color: #4CAF50;
  font-size: 14px;
  cursor: pointer;
  padding: 8px 0;
  transition: color 0.2s;
}

.back-link:hover {
  color: #45a049;
}

.new-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 32px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.page-title {
  font-size: 28px;
  color: #333;
  margin-bottom: 32px;
  text-align: center;
}

.question-form {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  color: #333;
  font-size: 14px;
  font-weight: 500;
}

.form-input,
.form-select {
  background-color: #fff;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 12px 16px;
  color: #333;
  font-size: 15px;
  font-family: inherit;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: #4CAF50;
}

.form-select {
  cursor: pointer;
}

.image-upload-area {
  width: 100%;
}

.upload-placeholder {
  background-color: #fafafa;
  border: 2px dashed #ddd;
  border-radius: 8px;
  height: 150px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.upload-placeholder:hover {
  border-color: #4CAF50;
  background-color: #f0f8f0;
}

.upload-icon {
  font-size: 32px;
}

.upload-text {
  color: #999;
  font-size: 14px;
}

.upload-preview {
  background-color: #fff;
  border: 2px solid #4CAF50;
  border-radius: 8px;
  height: 150px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 16px;
}

.preview-text {
  color: #333;
  font-size: 14px;
  text-align: center;
  word-break: break-all;
}

.remove-btn {
  padding: 6px 16px;
  background-color: #f44336;
  border: none;
  border-radius: 6px;
  color: #ffffff;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.remove-btn:hover {
  background-color: #d32f2f;
}

.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 8px;
}

.btn {
  flex: 1;
  padding: 14px;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  transition: all 0.2s;
  cursor: pointer;
}

.btn.cancel {
  background-color: #e0e0e0;
  color: #333;
}

.btn.cancel:hover {
  background-color: #d0d0d0;
}

.btn.submit {
  background-color: #4CAF50;
  color: #ffffff;
}

.btn.submit:hover {
  background-color: #45a049;
}
</style>
