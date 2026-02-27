// src/api/question.ts
import { call } from "./core";
import type { CreateQuestion } from "@/types/question";
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
  console.debug('createQuestion payload:', payload)
  // include both snake_case and camelCase keys because Tauri's generated
  // bindings may expect camelCase parameter names (e.g. questionImagePaths)
  const combined = {
    // snake_case
    ...payload,
    // camelCase variants
    questionImagePaths: payload.question_image_paths,
    answerImagePaths: payload.answer_image_paths,
    knowledgePoints: payload.knowledge_points,
  }
  // strip any reactive/prototype fields by serializing
  const plain = JSON.parse(JSON.stringify(combined))
  console.debug('createQuestion plain payload:', plain)
  return call<string>("create_question_comm", plain);
}

export function deleteQuestion(id: number) {
  return call<string>("delete_question_comm", { id });
}

export function restoreQuestion(id: number) {
  return call<string>("restore_question_comm", { id });
}

export function show_list_available_questions_page(page: number, pageSize: number) {
  return call<ActiveQuestion[]>("show_list_available_questions_page", {
    page,
    page_size: pageSize,
  });
}

export function show_list_deleted_questions_page(page: number, pageSize: number) {
  return call<DeleteQuestion[]>("show_list_deleted_questions_page", {
    page,
    page_size: pageSize,
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
    page_size: pageSize,
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
    page_size: pageSize,
  });
}
