// src/api/review.ts
import { call } from "./core";
import type { RecommendResult, ReviewResult } from "@/types/question";

// 新推荐系统的类型
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
}

/**
 * 获取每日推荐（新推荐系统）
 * @param targetCount 推荐数量，默认 10
 * @returns 每日推荐结果
 */
export function getDailyRecommendation(targetCount?: number) {
  return call<DailyRecommendation>("get_daily_recommendation_comm", {
    targetCount,
  });
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
 * @returns 推荐题目列表
 */
export function getRecommendationList(limit?: number) {
  return call<RecommendedQuestion[]>("get_recommendation_list_comm", {
    limit,
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
