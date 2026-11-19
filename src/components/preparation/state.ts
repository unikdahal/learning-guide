import { practiceQuestions, practiceTopics } from '../../data/preparation-practice.ts';
import type { PracticeQuestion, PracticeTopic } from '../../data/preparation-practice.ts';
export const STORAGE_KEY = 'learning-guide:preparation:v1';
export interface AnswerRecord { selected: number; attempts: number; correct: number; updatedAt: number }
export interface PracticeState { version: 1; answers: Record<string, AnswerRecord>; bookmarks: string[] }
export type ReviewMode = 'all' | 'mistakes' | 'bookmarks' | 'unanswered';
export const emptyState = (): PracticeState => ({ version: 1, answers: {}, bookmarks: [] });
const object = (v: unknown): v is Record<string, unknown> => !!v && typeof v === 'object' && !Array.isArray(v);
export function sanitizeState(raw: unknown): PracticeState {
  const state = emptyState();
  if (!object(raw) || raw.version !== 1) return state;
  for (const q of practiceQuestions) {
    const r = object(raw.answers) ? raw.answers[q.id] : undefined;
    if (!object(r)) continue;
    const { selected, attempts, correct, updatedAt } = r;
    if (typeof selected !== 'number' || !Number.isInteger(selected) || selected < 0 || selected > 3 ||
        typeof attempts !== 'number' || !Number.isSafeInteger(attempts) || attempts < 1 || attempts > 1_000_000 ||
        typeof correct !== 'number' || !Number.isSafeInteger(correct) || correct < 0 || correct > attempts ||
        typeof updatedAt !== 'number' || !Number.isSafeInteger(updatedAt) || updatedAt < 0 ||
        (selected === q.answer ? correct < 1 : correct >= attempts)) continue;
    state.answers[q.id] = { selected, attempts, correct, updatedAt };
  }
  if (Array.isArray(raw.bookmarks)) {
    const ids = new Set(practiceQuestions.map(q => q.id));
    state.bookmarks = [...new Set(raw.bookmarks.filter((id): id is string => typeof id === 'string' && ids.has(id)))];
  }
  return state;
}
export function parseState(value: string | null): PracticeState {
  try { return sanitizeState(value ? JSON.parse(value) : null); } catch { return emptyState(); }
}
export function recordAnswer(state: PracticeState, question: PracticeQuestion, selected: number, now = Date.now()): PracticeState {
  if (!Number.isInteger(selected) || selected < 0 || selected >= question.options.length) return state;
  const previous = state.answers[question.id];
  // At the storage cap, retain existing statistics rather than creating inconsistent counts.
  if (previous && previous.attempts >= 1_000_000) return state;
  return { ...state, answers: { ...state.answers, [question.id]: {
    selected, attempts: (previous?.attempts ?? 0) + 1,
    correct: (previous?.correct ?? 0) + Number(selected === question.answer), updatedAt: now,
  } } };
}
export function selectQuestions(state: PracticeState, topic: string, mode: ReviewMode, section = 'all', search = ''): PracticeQuestion[] {
  const query = search.trim().toLocaleLowerCase();
  return practiceQuestions.filter(q => (section === 'all' || q.section === section) &&
    (!query || `${q.prompt} NEC ${q.section}`.toLocaleLowerCase().includes(query)) && (topic === 'all' || q.topic === topic) && (
    mode === 'all' || (mode === 'bookmarks' && state.bookmarks.includes(q.id)) ||
    (mode === 'unanswered' && !state.answers[q.id]) ||
    (mode === 'mistakes' && !!state.answers[q.id] && state.answers[q.id].selected !== q.answer)));
}
export function topicPerformance(state: PracticeState): { topic: PracticeTopic; label: string; attempts: number; correct: number; seen: number; total: number }[] {
  return practiceTopics.map(([topic, label]) => {
    const questions = practiceQuestions.filter(q => q.topic === topic);
    const records = questions.map(q => state.answers[q.id]).filter((r): r is AnswerRecord => !!r);
    return { topic, label, total: questions.length, seen: records.length,
      attempts: records.reduce((sum, r) => sum + r.attempts, 0), correct: records.reduce((sum, r) => sum + r.correct, 0) };
  });
}
