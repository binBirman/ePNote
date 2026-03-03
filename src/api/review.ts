// src/api/review.ts
import { call } from "./core";
import type { RecommendResult, ReviewResult } from "@/types/question";

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
 * 获取推荐的复习题目
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
