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
  created_at: string;
  last_review: string;
}

export interface DeleteQuestion {
  id: number;
  subject: string;
  title: string;
  status: string;
  deleted_at: string;
}

// export enum QuestionState {
//   NEW,
//   LEARNING,
//   STABLE,
//   SUSPENDED,
// }
