export const STUDY_KEY = 'exam-preparation:coverage:v1';
export const DAY = 86_400_000;
export type Confidence = 'unseen' | 'learning' | 'review' | 'ready';
export interface TopicProgress { confidence: Confidence; written: boolean; updated: number }
export interface StudyState { version: 1; topics: Record<string, TopicProgress> }
const confidenceLevels: Confidence[] = ['unseen', 'learning', 'review', 'ready'];
export const emptyStudy = (): StudyState => ({ version: 1, topics: {} });

/** An import/storage boundary: never trust shape, topic IDs, or timestamps. */
export function parseStudy(raw: string | null, ids: readonly string[], now = Date.now()): StudyState {
  const state = emptyStudy();
  if (!raw || raw.length > 100_000) return state;
  try {
    const input: unknown = JSON.parse(raw);
    if (!input || typeof input !== 'object' || !('version' in input) || input.version !== 1 || !('topics' in input)) return state;
    const topics = input.topics;
    if (!topics || typeof topics !== 'object' || Array.isArray(topics)) return state;
    for (const id of ids) {
      if (!Object.hasOwn(topics, id)) continue;
      const row: unknown = (topics as Record<string, unknown>)[id];
      if (!row || typeof row !== 'object') continue;
      const p = row as Partial<TopicProgress>;
      if (!confidenceLevels.includes(p.confidence as Confidence) || typeof p.written !== 'boolean' ||
          typeof p.updated !== 'number' || !Number.isFinite(p.updated) || p.updated < 0 || p.updated > now + DAY) continue;
      state.topics[id] = { confidence: p.confidence as Confidence, written: p.written, updated: Math.min(p.updated, now) };
    }
  } catch { /* A corrupt browser save should never break the syllabus. */ }
  return state;
}

/** A transparent revision aid, not a prediction of marks or spaced-repetition research. */
export function reviewDue(progress: TopicProgress | undefined, now = Date.now()): boolean {
  if (!progress || progress.confidence === 'unseen') return false;
  const days = progress.confidence === 'ready' ? 7 : progress.confidence === 'review' ? 1 : 3;
  return now - progress.updated >= days * DAY;
}

export function priority(progress: TopicProgress | undefined, shared: boolean, now = Date.now()): number {
  const confidence = progress?.confidence ?? 'unseen';
  return ({ unseen: 40, learning: 30, review: 60, ready: 0 }[confidence]) +
    (reviewDue(progress, now) ? 100 : 0) + (shared ? 10 : 0);
}
