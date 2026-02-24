<script setup lang="ts">
import { computed } from 'vue'
import { mockStats, getQuestions } from '../mock/data'

const questions = getQuestions()

// 扩展统计信息
const todayReviewed = computed(() => 2) // Mock 今日已复习数
const totalReviews = computed(() => 15) // Mock 总复习次数
const averageAccuracy = computed(() => 73) // Mock 平均准确率

// 按状态统计的题目数
const stateStats = computed(() => {
  return {
    NEW: questions.filter(q => q.state === 'NEW').length,
    LEARNING: questions.filter(q => q.state === 'LEARNING').length,
    STABLE: questions.filter(q => q.state === 'STABLE').length
  }
})

// 计算进度百分比
const getProgressWidth = (count: number) => {
  const total = questions.length
  return total > 0 ? (count / total * 100).toFixed(1) : 0
}
</script>

<template>
  <div class="stats-container">
    <h1 class="page-title">统计</h1>

    <!-- 概览卡片 -->
    <div class="overview-cards">
      <div class="stat-card">
        <div class="stat-icon">📚</div>
        <div class="stat-info">
          <div class="stat-value">{{ mockStats.totalQuestions }}</div>
          <div class="stat-label">总题目数</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">🎯</div>
        <div class="stat-info">
          <div class="stat-value">{{ todayReviewed }}</div>
          <div class="stat-label">今日已复习</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">🔄</div>
        <div class="stat-info">
          <div class="stat-value">{{ totalReviews }}</div>
          <div class="stat-label">总复习次数</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">📊</div>
        <div class="stat-info">
          <div class="stat-value">{{ averageAccuracy }}%</div>
          <div class="stat-label">平均准确率</div>
        </div>
      </div>
    </div>

    <!-- 题目状态分布 -->
    <div class="section-card">
      <h2 class="section-title">题目状态分布</h2>
      <div class="state-distribution">
        <div class="state-item">
          <div class="state-header">
            <span class="state-name new">新题</span>
            <span class="state-count">{{ stateStats.NEW }} 题</span>
          </div>
          <div class="progress-bar">
            <div
              class="progress-fill new"
              :style="{ width: getProgressWidth(stateStats.NEW) + '%' }"
            ></div>
          </div>
          <div class="state-percent">{{ getProgressWidth(stateStats.NEW) }}%</div>
        </div>

        <div class="state-item">
          <div class="state-header">
            <span class="state-name learning">学习中</span>
            <span class="state-count">{{ stateStats.LEARNING }} 题</span>
          </div>
          <div class="progress-bar">
            <div
              class="progress-fill learning"
              :style="{ width: getProgressWidth(stateStats.LEARNING) + '%' }"
            ></div>
          </div>
          <div class="state-percent">{{ getProgressWidth(stateStats.LEARNING) }}%</div>
        </div>

        <div class="state-item">
          <div class="state-header">
            <span class="state-name stable">已掌握</span>
            <span class="state-count">{{ stateStats.STABLE }} 题</span>
          </div>
          <div class="progress-bar">
            <div
              class="progress-fill stable"
              :style="{ width: getProgressWidth(stateStats.STABLE) + '%' }"
            ></div>
          </div>
          <div class="state-percent">{{ getProgressWidth(stateStats.STABLE) }}%</div>
        </div>
      </div>
    </div>

    <!-- 今日复习情况 -->
    <div class="section-card">
      <h2 class="section-title">今日复习情况</h2>
      <div class="today-stats">
        <div class="today-item">
          <div class="today-label">待复习</div>
          <div class="today-value">{{ mockStats.todayPending }}</div>
        </div>
        <div class="today-item">
          <div class="today-label">已完成</div>
          <div class="today-value completed">{{ todayReviewed }}</div>
        </div>
      </div>
      <div class="today-tip">
        💡 继续加油！完成今天的复习计划。
      </div>
    </div>
  </div>
</template>

<style scoped>
.stats-container {
  max-width: 800px;
}

.page-title {
  font-size: 28px;
  color: #333;
  margin-bottom: 24px;
  text-align: center;
}

/* 概览卡片 */
.overview-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.stat-icon {
  font-size: 32px;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 28px;
  font-weight: bold;
  color: #333;
  line-height: 1.2;
}

.stat-label {
  color: #666;
  font-size: 13px;
  margin-top: 4px;
}

/* 区块卡片 */
.section-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.section-title {
  font-size: 18px;
  color: #333;
  margin-bottom: 20px;
  font-weight: 600;
}

/* 状态分布 */
.state-distribution {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.state-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.state-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.state-name {
  font-size: 14px;
  font-weight: 500;
}

.state-name.new {
  color: #2196F3;
}

.state-name.learning {
  color: #FF9800;
}

.state-name.stable {
  color: #4CAF50;
}

.state-count {
  color: #666;
  font-size: 14px;
}

.progress-bar {
  height: 8px;
  background-color: #f0f0f0;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.progress-fill.new {
  background-color: #2196F3;
}

.progress-fill.learning {
  background-color: #FF9800;
}

.progress-fill.stable {
  background-color: #4CAF50;
}

.state-percent {
  text-align: right;
  color: #666;
  font-size: 13px;
}

/* 今日复习情况 */
.today-stats {
  display: flex;
  gap: 24px;
  margin-bottom: 16px;
}

.today-item {
  flex: 1;
  text-align: center;
  background-color: #fafafa;
  border-radius: 8px;
  padding: 20px;
}

.today-label {
  color: #666;
  font-size: 14px;
  margin-bottom: 12px;
}

.today-value {
  font-size: 36px;
  font-weight: bold;
  color: #333;
}

.today-value.completed {
  color: #4CAF50;
}

.today-tip {
  color: #666;
  font-size: 14px;
  padding: 12px;
  background-color: #fafafa;
  border-radius: 8px;
  border-left: 3px solid #4CAF50;
}
</style>
