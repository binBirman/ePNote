<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { getStats, getDailyReviewStatus, type StatsData, type DailyReviewStatus } from '@/api/review'

const stats = ref<StatsData | null>(null)
const dailyStatus = ref<DailyReviewStatus | null>(null)
const loading = ref(true)

onMounted(async () => {
  try {
    // 获取基础统计数据
    stats.value = await getStats()

    // 获取今日复习状态
    dailyStatus.value = await getDailyReviewStatus()
  } catch (e) {
    console.error('加载统计失败:', e)
  } finally {
    loading.value = false
  }
})

// 按状态统计的题目数
const stateStats = computed(() => {
  if (!stats.value) return { NEW: 0, LEARNING: 0, STABLE: 0 }
  return {
    NEW: stats.value.state_counts.new_count,
    LEARNING: stats.value.state_counts.learning_count,
    STABLE: stats.value.state_counts.stable_count
  }
})

// 今日待复习数（使用更准确的推荐系统数据）
const todayPending = computed(() => {
  if (!dailyStatus.value) return stats.value?.today_pending || 0
  return dailyStatus.value.recommended_count - dailyStatus.value.reviewed_count
})

// 今日已完成数（使用更准确的推荐系统数据）
const todayCompleted = computed(() => {
  if (!dailyStatus.value) return stats.value?.today_reviewed || 0
  return dailyStatus.value.reviewed_count
})

// 计算进度百分比
const getProgressWidth = (count: number) => {
  if (!stats.value || stats.value.total_questions === 0) return '0'
  return (count / stats.value.total_questions * 100).toFixed(1)
}
</script>

<template>
  <div class="stats-container">
    <h1 class="page-title">统计</h1>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading">加载中...</div>

    <template v-else-if="stats">
    <!-- 概览卡片 -->
    <div class="overview-cards">
      <div class="stat-card">
        <div class="stat-icon icon-blue">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="3" y="4" width="18" height="5" rx="0.8" />
            <rect x="3" y="11" width="18" height="5" rx="0.8" />
            <rect x="3" y="18" width="18" height="5" rx="0.8" />
            <line x1="6" y1="6.5" x2="14" y2="6.5" />
            <line x1="6" y1="13.5" x2="14" y2="13.5" />
            <line x1="6" y1="20.5" x2="14" y2="20.5" />
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.total_questions }}</div>
          <div class="stat-label">总题目数</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon icon-green">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="9" />
            <circle cx="12" cy="12" r="5.5" />
            <circle cx="12" cy="12" r="2" />
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.today_reviewed }}</div>
          <div class="stat-label">今日已复习</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon icon-orange">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M20 12a8 8 0 0 1 -14 5.3" />
            <polyline points="6 21 6 16 11 16" />
            <path d="M4 12a8 8 0 0 1 14 -5.3" />
            <polyline points="18 3 18 8 13 8" />
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.total_reviews }}</div>
          <div class="stat-label">总复习次数</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon icon-purple">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <line x1="5" y1="20" x2="20" y2="20" />
            <rect x="6" y="14" width="3.5" height="6" />
            <rect x="11" y="9" width="3.5" height="11" />
            <rect x="16" y="4" width="3.5" height="16" />
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.average_accuracy.toFixed(0) }}%</div>
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
          <div class="today-value">{{ todayPending }}</div>
        </div>
        <div class="today-item">
          <div class="today-label">已完成</div>
          <div class="today-value completed">{{ todayCompleted }}</div>
        </div>
      </div>
      <div class="today-tip">
        <template v-if="todayPending <= 0">
          🎉 恭喜！今日复习计划已完成！
        </template>
        <template v-else>
          💡 继续加油！还剩 {{ todayPending }} 题未复习。
        </template>
      </div>
    </div>
    </template>
  </div>
</template>

<style scoped>
.stats-container {
  max-width: 800px;
  width: 100%;
  margin-left: 0;
}

.loading {
  text-align: center;
  color: #666;
  font-size: 16px;
  padding: 40px;
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
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-icon svg {
  width: 100%;
  height: 100%;
}

.icon-blue {
  color: #2196F3;
}

.icon-green {
  color: #4CAF50;
}

.icon-orange {
  color: #FF9800;
}

.icon-purple {
  color: #9C27B0;
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
