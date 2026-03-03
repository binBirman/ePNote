// src/types/question.ts

export interface CreateQuestion {
  name: string;
  question_image_paths: string[];
  answer_image_paths: string[];
  subject: string;
  knowledge_points: string[];
}

export interface ActiveQuestion {
  id: number;
  subject: string;
  title: string;
  status: string;
  knowledge_points: string[];
  created_at: string;
  last_review: string;
}

export interface DeleteQuestion {
  id: number;
  subject: string;
  title: string;
  status: string;
  knowledge_points: string[];
  deleted_at: string;
}

export interface QuestionImage {
  path: string;
  asset_id?: string;
}

export interface QuestionInfo {
  id: number;
  name: string | null;
  state: string;
  created_at: string;
  deleted_at: string | null;
  subject: string | null;
  knowledge_points: string[];
  question_images: QuestionImage[];
  answer_images: QuestionImage[];
  last_reviewed_at: string | null;
}

export type QuestionState = 'NEW' | 'LEARNING' | 'STABLE' | 'DUE' | 'SUSPENDED';

// 复习结果类型
export type ReviewResult = 'CORRECT' | 'WRONG' | 'FUZZY';

// 推荐结果
export interface RecommendResult {
  questions: RecommendQuestion[];
  reasons: string[];
  subject: string | null;
}

// 推荐题目
export interface RecommendQuestion {
  id: number;
  name: string | null;
  state: string;
  created_at: string;
  last_review_at: string | null;
  correct_streak: number;
  wrong_count: number;
  due_at: string | null;
}

// export enum QuestionState {
//   NEW,
//   LEARNING,
//   STABLE,
//   SUSPENDED,
// }
