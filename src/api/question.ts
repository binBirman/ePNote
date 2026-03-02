// src/api/question.ts
import { call } from "./core";
import type { CreateQuestion, QuestionInfo } from "@/types/question";
import type { ActiveQuestion, DeleteQuestion } from "@/types/question";

export function createQuestion(context: CreateQuestion) {
  const payload = {
    name: context.name,
    question_image_paths: context.question_image_paths,
    answer_image_paths: context.answer_image_paths,
    // backend accepts Option<String> for subject
    subject: context.subject,
    knowledge_points: context.knowledge_points,
  }
  console.log('[DEBUG] createQuestion payload:', payload)
  // Tauri expects camelCase parameter names
  const args = {
    name: context.name,
    questionImagePaths: context.question_image_paths,
    answerImagePaths: context.answer_image_paths,
    subject: context.subject,
    knowledgePoints: context.knowledge_points,
  }
  console.log('[DEBUG] createQuestion args (questionImagePaths):', args.questionImagePaths)
  console.log('[DEBUG] createQuestion args (answerImagePaths):', args.answerImagePaths)
  return call<string>("create_question_comm", args);
}

export function deleteQuestion(id: number) {
  return call<string>("delete_question_comm", { id });
}

export function restoreQuestion(id: number) {
  return call<string>("restore_question_comm", { id });
}

export function getQuestionData(id: number) {
  return call<QuestionInfo>("get_question_detail_comm", { id });
}

export function show_list_available_questions_page(page: number, pageSize: number) {
  console.log("发送参数:", { page, page_size: pageSize })
  return call<ActiveQuestion[]>("show_list_available_questions_page", {
    page,
    pageSize,
  });
}

export function show_list_deleted_questions_page(page: number, pageSize: number) {
  return call<DeleteQuestion[]>("show_list_deleted_questions_page", {
    page,
    pageSize,
  });
}

export function show_list_available_questions_by_state_page(
  question_state: string,
  page: number,
  pageSize: number
) {
  return call<ActiveQuestion[]>("show_list_available_questions_by_state_page", {
    question_state,
    page,
    pageSize,
  });
}

export function show_list_available_questions_by_subject_page(
  subject: string,
  page: number,
  pageSize: number
) {
  return call<ActiveQuestion[]>("show_list_available_questions_by_subject_page", {
    subject,
    page,
    pageSize,
  });
}

export function show_list_available_questions_by_subject_and_state_page(
  subject: string,
  questionState: string,
  page: number,
  pageSize: number
) {
  return call<ActiveQuestion[]>("show_list_available_questions_by_subject_and_state_page", {
    subject,
    questionState,
    page,
    pageSize,
  });
}

export function show_subjects() {
  return call<string[]>("show_subjects");
}

export function show_states() {
  return call<string[]>("show_states");
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
