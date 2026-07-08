// src/api/review.ts
import { call } from "./core";
import type { RecommendResult, ReviewResult } from "@/types/question";

// 新推荐系统的类型

export interface ScoreDetail {
  forget_risk: number
  freshness_bonus: number
  last_wrong_bonus: number
  error_rate_bonus: number
  randomness: number
  final_score: number
}

export interface RecommendedQuestion {
  question_id: number
  name: string | null
  score: number
  state: string
  due_at: number | null
  correct_streak: number
  wrong_count: number
  last_result: string | null
  error_rate: number | null
  subject: string | null
  reason?: string[]
  score_detail?: ScoreDetail
}

export interface DailyRecommendation {
  day: number
  questions: RecommendedQuestion[]
}

export interface DailyReviewStatus {
  recommended_count: number
  reviewed_count: number
  is_completed: boolean
}

export interface ReviewRecord {
  question_id: number
  question_name: string | null
  result: string
  reviewed_at: number
  subject: string | null
}

export interface PreviewRecommendationItem {
  question_id: number
  name: string
  subject: string | null
  score: number
  selected: boolean
  reason: string[]
  exclusion_reason: string[]
  score_detail: ScoreDetail | null
  subject_rank: number
  subject_limit: number
}

export interface RecommendationStats {
  total_questions: number
  participating_questions: number
  archived_subjects: string[]
  recommended_count: number
  new_questions: number
  pending_review: number
}

/**
 * 获取每日推荐（新推荐系统）
 * @returns 每日推荐结果
 */
export function getDailyRecommendation() {
  return call<DailyRecommendation>("get_daily_recommendation_comm", {});
}

/**
 * 获取今日复习状态
 * @returns 复习状态（推荐数、已复习数、是否完成）
 */
export function getDailyReviewStatus() {
  return call<DailyReviewStatus>("get_daily_review_status_comm", {});
}

/**
 * 获取今日复习记录（用于总结页面）
 * @returns 今日复习记录列表
 */
export function getTodayReviewRecords() {
  return call<ReviewRecord[]>("get_today_review_records_comm", {});
}

/**
 * 获取推荐题目列表（新推荐系统）
 * @param limit 推荐数量，默认 10
 * @param subject 可选的科目筛选；"全部"/不传 返回所有科目
 * @returns 推荐题目列表
 */
export function getRecommendationList(limit?: number, subject?: string) {
  return call<RecommendedQuestion[]>("get_recommendation_list_comm", {
    limit,
    subject,
  });
}

/**
 * 处理复习结果
 * @param questionId 题目ID
 * @param result 复习结果：CORRECT, WRONG, FUZZY
 * @returns 更新后的题目信息
 */
export function processReview(questionId: number, result: ReviewResult) {
  return call("process_review_comm", {
    questionId,
    result,
  });
}

/**
 * 获取推荐的复习题目（旧接口，兼容用）
 * @param limit 推荐数量，默认 10
 * @param subject 可选的科目筛选
 * @returns 推荐结果
 */
export function recommendQuestions(limit?: number, subject?: string) {
  return call<RecommendResult>("recommend_questions_comm", {
    limit,
    subject,
  });
}

/**
 * 暂停题目
 * @param questionId 题目ID
 * @returns 更新后的题目信息
 */
export function suspendQuestion(questionId: number) {
  return call("suspend_question_comm", {
    questionId,
  });
}

/**
 * 恢复题目
 * @param questionId 题目ID
 * @returns 更新后的题目信息
 */
export function recoverQuestion(questionId: number) {
  return call("recover_question_comm", {
    questionId,
  });
}

/**
 * 获取所有科目列表
 * @returns 科目列表
 */
export function listSubjects() {
  return call<string[]>("list_subjects_comm", {});
}

/**
 * 统计结果
 */
export interface StatsData {
  total_questions: number;
  today_reviewed: number;
  total_reviews: number;
  correct_count: number;
  wrong_count: number;
  fuzzy_count: number;
  state_counts: {
    new_count: number;
    learning_count: number;
    stable_count: number;
    due_count: number;
    suspended_count: number;
  };
  today_pending: number;
  average_accuracy: number;
}

/**
 * 获取统计信息
 * @returns 统计数据
 */
export function getStats() {
  return call<StatsData>("get_stats_comm", {});
}

/**
 * 根据题目ID列表获取题目（用于练习模式）
 * @param questionIds 题目ID列表
 * @returns 题目列表
 */
export function getQuestionsByIds(questionIds: number[]) {
  return call<RecommendedQuestion[]>("get_questions_by_ids_comm", {
    questionIds,
  });
}

/**
 * 预览推荐（展示全部题目的评分和入选状态，不写库）
 * @param showScoreDetail 是否显示评分详情
 * @param showExclusionReason 是否显示落选原因
 */
export function previewRecommendation(showScoreDetail: boolean, showExclusionReason: boolean) {
  return call<PreviewRecommendationItem[]>("preview_recommendation_comm", {
    showScoreDetail,
    showExclusionReason,
  });
}

/**
 * 重新生成今日推荐（删除缓存后重新生成）
 */
export function regenerateDailyRecommendation() {
  return call<DailyRecommendation>("regenerate_daily_recommendation_comm", {});
}

/**
 * 获取推荐统计概览
 */
export function getRecommendationStats() {
  return call<RecommendationStats>("get_recommendation_stats_comm", {});
}
