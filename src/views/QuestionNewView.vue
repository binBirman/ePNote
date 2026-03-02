<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { createQuestion } from '@/api/question'
import { open } from "@tauri-apps/plugin-dialog";

const router = useRouter()

const questionName = ref('')
const questionSubject = ref('')
const questionKnowledgePoint = ref('')
const questionQuestionImages = ref<string[]>([])
const questionAnswerImages = ref<string[]>([])
const loading = ref(false)

const handleSubmit = async () => {
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

  if (questionQuestionImages.value.length === 0) {
    alert('请至少选择一张题目图片')
    return
  }

  if (questionAnswerImages.value.length === 0) {
    alert('请至少选择一张答案图片')
    return
  }

  try {
    console.log('handleSubmit start')
    console.log('question_image_paths:', questionQuestionImages.value)
    console.log('answer_image_paths:', questionAnswerImages.value)
    loading.value = true
    await createQuestion({
      name: questionName.value,
      subject: questionSubject.value,
      knowledge_points: questionKnowledgePoint.value
        ? questionKnowledgePoint.value.split(',').map(s => s.trim()).filter(Boolean)
        : [],
      question_image_paths: questionQuestionImages.value,
      answer_image_paths: questionAnswerImages.value,
    })

    // 清空表单
  questionName.value = ''
  questionSubject.value = ''
  questionKnowledgePoint.value = ''
  questionQuestionImages.value = []
  questionAnswerImages.value = []

    alert('题目创建成功！')
    router.push({ path: '/questions', query: { r: String(Date.now()) } })
  } catch (e) {
    console.error(e)
    alert('创建题目失败：' + String(e))
  }

  finally {
    loading.value = false
    console.log('handleSubmit end')
  }

}

const handleCancel = () => {
  if (questionName.value || questionSubject.value || questionKnowledgePoint.value ||
      questionQuestionImages.value.length > 0 || questionAnswerImages.value.length > 0) {
    if (!confirm('确定要放弃当前编辑吗？')) {
      return
    }
  }
  router.push('/questions')
}

// 使用 Tauri dialog 选择多个图片（逐个选择，可以多次添加）
const selectQuestionImages = async () => {
  try {
    const res = await open({
      multiple: true,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }]
    })
    if (res) {
      // Tauri dialog 返回字符串数组或单个字符串
      const paths = Array.isArray(res) ? res : [res]
      console.log('[DEBUG] Selected question images:', paths)
      // 追加到现有图片列表
      questionQuestionImages.value = [...questionQuestionImages.value, ...paths.map(p => p as string)]
    }
  } catch (e) {
    console.error('Failed to select images:', e)
    alert('选择图片失败: ' + e)
  }
}

const selectAnswerImages = async () => {
  try {
    const res = await open({
      multiple: true,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }]
    })
    if (res) {
      const paths = Array.isArray(res) ? res : [res]
      console.log('[DEBUG] Selected answer images:', paths)
      questionAnswerImages.value = [...questionAnswerImages.value, ...paths.map(p => p as string)]
    }
  } catch (e) {
    console.error('Failed to select images:', e)
    alert('选择图片失败: ' + e)
  }
}

const removeQuestionImage = (index: number) => {
  questionQuestionImages.value.splice(index, 1)
}

const removeAnswerImage = (index: number) => {
  questionAnswerImages.value.splice(index, 1)
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
          <label class="form-label">题目图 * (可多选)</label>
          <div class="image-upload-area">
            <div v-if="questionQuestionImages.length === 0" class="upload-placeholder" @click="selectQuestionImages">
              <span class="upload-icon">📷</span>
              <span class="upload-text">点击选择题目图片</span>
            </div>
            <div v-else class="upload-preview-list">
              <div v-for="(img, index) in questionQuestionImages" :key="'q'+index" class="upload-preview-item">
                <span class="preview-text">{{ img }}</span>
                <button type="button" class="remove-btn" @click.stop="removeQuestionImage(index)">删除</button>
              </div>
              <button type="button" class="add-more-btn" @click.stop="selectQuestionImages">+ 添加更多</button>
            </div>
          </div>
        </div>

        <!-- 答案图 -->
        <div class="form-group">
          <label class="form-label">答案图 * (可多选)</label>
          <div class="image-upload-area">
            <div v-if="questionAnswerImages.length === 0" class="upload-placeholder" @click="selectAnswerImages">
              <span class="upload-icon">📷</span>
              <span class="upload-text">点击选择答案图片</span>
            </div>
            <div v-else class="upload-preview-list">
              <div v-for="(img, index) in questionAnswerImages" :key="'a'+index" class="upload-preview-item">
                <span class="preview-text">{{ img }}</span>
                <button type="button" class="remove-btn" @click.stop="removeAnswerImage(index)">删除</button>
              </div>
              <button type="button" class="add-more-btn" @click.stop="selectAnswerImages">+ 添加更多</button>
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
  width: 100%;
  max-width: 800px;
  margin: 0 auto;
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

.upload-preview-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.upload-preview-item {
  background-color: #fff;
  border: 2px solid #4CAF50;
  border-radius: 8px;
  padding: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.upload-preview-item .preview-text {
  flex: 1;
  text-align: left;
}

.add-more-btn {
  padding: 8px 16px;
  background-color: #f0f0f0;
  border: 2px dashed #ccc;
  border-radius: 8px;
  color: #666;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.add-more-btn:hover {
  border-color: #4CAF50;
  color: #4CAF50;
  background-color: #f8f8f8;
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
