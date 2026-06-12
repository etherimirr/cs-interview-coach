import { invoke } from '@tauri-apps/api/core';
import type {
  Job,
  KnowledgeCard,
  ReviewCard,
  Slots,
  Suggestion,
  Taxonomy,
  Grade,
} from './types';

export const api = {
  // Taxonomy
  getTaxonomy: () => invoke<Taxonomy>('get_taxonomy'),

  // KnowledgeCard
  createCard: (title: string, topic_ids: string[]) =>
    invoke<KnowledgeCard>('create_card', { title, topicIds: topic_ids }),
  getCard: (id: string) =>
    invoke<KnowledgeCard | null>('get_card', { id }),
  listCards: () => invoke<KnowledgeCard[]>('list_cards'),
  listCardsByTopic: (topic_id: string) =>
    invoke<KnowledgeCard[]>('list_cards_by_topic', { topicId: topic_id }),
  updateCardSlots: (id: string, slots: Slots, aliases: string[]) =>
    invoke<KnowledgeCard>('update_card_slots', { id, slots, aliases }),
  deleteCard: (id: string) => invoke<void>('delete_card', { id }),

  // Suggest
  suggestTitles: (prefix: string, limit = 10) =>
    invoke<Suggestion[]>('suggest_titles', { prefix, limit }),

  // ReviewCard
  createReview: (knowledge_card_id: string, question: string, answer: string) =>
    invoke<ReviewCard>('create_review', { knowledgeCardId: knowledge_card_id, question, answer }),
  listReviewsForCard: (knowledge_card_id: string) =>
    invoke<ReviewCard[]>('list_reviews_for_card', { knowledgeCardId: knowledge_card_id }),
  dueReviews: () => invoke<ReviewCard[]>('due_reviews'),
  rateReview: (id: string, grade: Grade) =>
    invoke<ReviewCard>('rate_review', { id, grade }),

  // Jobs
  listJobs: () => invoke<Job[]>('list_jobs'),
  getJob: (id: string) => invoke<Job | null>('get_job', { id }),
  listCardsForJob: (id: string) =>
    invoke<KnowledgeCard[]>('list_cards_for_job', { id }),
};
