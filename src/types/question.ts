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

// export enum QuestionState {
//   NEW,
//   LEARNING,
//   STABLE,
//   SUSPENDED,
// }
