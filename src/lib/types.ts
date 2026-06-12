// TS mirrors of Rust structs in src-tauri/src/models.rs and taxonomy.rs.
// Keep in sync.

export interface Cite {
  source_id: string;
  locator: string;
}

export interface Fact {
  text: string;
  cites: Cite[];
}

export interface Slots {
  definition: Fact[];
  mechanism: Fact[];
  complexity: Fact[];
  comparison: Fact[];
  use_cases: Fact[];
  interview_points: Fact[];
  pitfalls: Fact[];
  code: Fact[];
}

export const EMPTY_SLOTS: Slots = {
  definition: [],
  mechanism: [],
  complexity: [],
  comparison: [],
  use_cases: [],
  interview_points: [],
  pitfalls: [],
  code: [],
};

export const SLOT_LABELS: Record<keyof Slots, string> = {
  definition: '定义',
  mechanism: '原理机制',
  complexity: '复杂度',
  comparison: '对比相关',
  use_cases: '应用场景',
  interview_points: '经典考点',
  pitfalls: '易错点',
  code: '代码示例',
};

export type QuestionDimension = 'what' | 'why' | 'how' | 'when' | 'pitfall' | 'extension';

export interface QuestionNode {
  dimension: QuestionDimension;
  question: string;
  answer_outline: string;
  children: QuestionNode[];
}

export interface KnowledgeCard {
  id: string;
  title: string;
  aliases: string[];
  topic_ids: string[];
  related_card_ids: string[];
  slots: Slots;
  question_tree: QuestionNode[];
  created_at: string;
  updated_at: string;
}

export interface FsrsState {
  stability: number;
  difficulty: number;
  last_review: string;
  next_review: string;
  reps: number;
  lapses: number;
}

export interface ReviewCard {
  id: string;
  knowledge_card_id: string;
  question: string;
  answer: string;
  fsrs: FsrsState;
  created_at: string;
}

export type Grade = 'again' | 'hard' | 'good' | 'easy';

export interface Suggestion {
  key: string;
  card_id: string;
}

// Jobs (seed/jobs.yaml)
export interface Job {
  id: string;
  title: string;
  company: string;
  location: string;
  level: string;
  track: string;
  jd: string;
  hard_requirements: string[];
  relevant_topic_ids: string[];
  cherry_picked_cards: string[];
  my_anchors: string[];
  notes: string;
}

// Taxonomy — id can be int or float-like ("1.1") from YAML
export interface SubTopic {
  id: number | string;
  name: string;
  hint?: string;
}

export interface Topic {
  id: number | string;
  name: string;
  short?: string;
  children: SubTopic[];
}

export interface Group {
  id: string;
  name: string;
  topics: number[];
}

export interface Taxonomy {
  version: string;
  locked_levels: number;
  groups: Group[];
  topics: Topic[];
}
