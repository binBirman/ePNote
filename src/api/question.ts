// src/api/question.ts
import { call } from "./core";
import type { CreateQuestion, QuestionInfo } from "@/types/question";
import type { ActiveQuestion, DeleteQuestion } from "@/types/question";

export function createQuestion(context: CreateQuestion) {
  const args = {
    name: context.name,
    questionImagePaths: context.question_image_paths,
    answerImagePaths: context.answer_image_paths,
    subject: context.subject,
    knowledgePoints: context.knowledge_points,
  }
  return call<string>("create_question_comm", args);
}

export function deleteQuestion(id: number) {
  return call<string>("delete_question_comm", { id });
}

export function restoreQuestion(id: number) {
  return call<string>("restore_question_comm", { id });
}

export function permanentlyDeleteQuestion(id: number) {
  return call<string>("permanently_delete_question_comm", { id });
}

export function cleanupRecycleBin(daysThreshold?: number) {
  return call<number>("cleanup_recycle_bin_comm", { days_threshold: daysThreshold });
}

export function getQuestionData(id: number) {
  return call<QuestionInfo>("get_question_detail_comm", { id });
}

/**
 * 分类查询：按科目 + 状态分页过滤；两者均可不传（"ALL" 或 null）。
 * page 为 0-indexed。
 */
export function classifyQuestions(params: {
  subject?: string | null
  questionState?: string | null
  page: number
  pageSize: number
}): Promise<ActiveQuestion[]> {
  return call<ActiveQuestion[]>("classify_questions", {
    subject: params.subject ?? null,
    questionState: params.questionState ?? null,
    page: params.page,
    pageSize: params.pageSize,
  })
}

/**
 * 搜索查询：单字符串输入。
 * - 后端能解析为 i64 → 题目 ID 精确查询；
 * - 否则 → 名称 / 知识点模糊匹配。
 * 可叠加 subject / questionState 进行二次过滤（Mode B）。
 */
export function searchQuestions(params: {
  query: string
  subject?: string | null
  questionState?: string | null
  page: number
  pageSize: number
}): Promise<ActiveQuestion[]> {
  return call<ActiveQuestion[]>("search_questions", {
    query: params.query,
    subject: params.subject ?? null,
    questionState: params.questionState ?? null,
    page: params.page,
    pageSize: params.pageSize,
  })
}

export function show_list_deleted_questions_page(page: number, pageSize: number) {
  return call<DeleteQuestion[]>("show_list_deleted_questions_page", {
    page,
    pageSize,
  });
}

export function show_subjects() {
  return call<string[]>("show_subjects");
}

export interface UpdateQuestionInput {
  name?: string;
  subject?: string;
  knowledge_points?: string[];
}

export function updateQuestion(id: number, data: UpdateQuestionInput) {
  return call<string>("update_question_comm", {
    id,
    data: {
      name: data.name,
      subject: data.subject,
      knowledge_points: data.knowledge_points,
    }
  });
}

export function getImageBase64(path: string) {
  return call<string>("get_image_base64", { path });
}

export function addQuestionImages(id: number, imagePaths: string[], imageType: string) {
  return call<string>("add_question_images_comm", {
    id,
    imagePaths,
    imageType,
  });
}

export function deleteQuestionImage(assetId: string) {
  return call<string>("delete_question_image_comm", { assetId });
}

export function updateImageSortOrder(questionId: number, type: string, updates: { asset_id: string; sort_order: number }[]) {
  return call<string>("update_image_sort_order_comm", { questionId, type, updates });
}