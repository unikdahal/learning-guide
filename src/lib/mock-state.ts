import { practiceQuestions } from '../data/preparation-practice.ts';
import { parseState, recordAnswer, STORAGE_KEY, type PracticeState } from '../components/preparation/state.ts';

/** Backup integration: optional envelope `mock: MockState`. Read with parseMock(raw, now),
 * export with exportMock(state, now), restore JSON.stringify(parsed) at MOCK_KEY alongside
 * the matching practice namespace. Preserve pending journals; do not merge sessions or
 * replay their answers on import. Missing namespace means emptyMock(). Parsing is pure. */
export const MOCK_KEY = 'exam-preparation:mock:v1';
export interface MockSession {
  id: string; seed: string; kind: 'round' | 'retry'; ids: string[];
  started: number; deadline: number; penalty: 0 | 0.2;
  answers: Record<string, number>; flagged: string[]; cursor: number;
  finished: number | null; reason: 'manual' | 'expiry' | null; recorded: boolean;
}
export interface MockState {
  version: 1; session: MockSession | null;
  /** Write-ahead journal: persisted before practice history is changed. */
  pending: { sessionId: string; before: PracticeState; after: PracticeState } | null;
}
export interface MockStorage { getItem(key: string): string | null; setItem(key: string, value: string): void }
export const emptyMock = (): MockState => ({ version: 1, session: null, pending: null });
const bank = new Map(practiceQuestions.map(q => [q.id, q]));
const object = (v: unknown): v is Record<string, unknown> => !!v && typeof v === 'object' && !Array.isArray(v);
const time = (v: unknown): v is number => typeof v === 'number' && Number.isSafeInteger(v) && v >= 0;
function random(seed: string) {
  let x = 2166136261;
  for (const ch of seed) x = Math.imul(x ^ ch.charCodeAt(0), 16777619);
  return () => { x += 0x6D2B79F5; let n = x; n = Math.imul(n ^ n >>> 15, n | 1); n ^= n + Math.imul(n ^ n >>> 7, n | 61); return ((n ^ n >>> 14) >>> 0) / 4294967296; };
}
function shuffle<T>(values: readonly T[], rng: () => number): T[] {
  const result = [...values];
  for (let i = result.length - 1; i > 0; i--) { const j = Math.floor(rng() * (i + 1)); [result[i], result[j]] = [result[j]!, result[i]!]; }
  return result;
}
/** Equal chapter quotas; first pass visits distinct sections, second pass uses their other question. */
export function selectMockQuestions(count: 20 | 60 | 100, seed: string): string[] {
  if (![20, 60, 100].includes(count) || !seed || seed.length > 100) throw new RangeError('Choose 20, 60 or 100 questions and a seed of 1–100 characters.');
  const rng = random(seed), picked: string[] = [];
  for (let chapter = 1; chapter <= 10; chapter++) {
    const sections = shuffle([...new Set(practiceQuestions.filter(q => q.section.startsWith(`${chapter}.`)).map(q => q.section))], rng);
    const groups = sections.map(section => shuffle(practiceQuestions.filter(q => q.section === section), rng));
    const ordered = [0, 1].flatMap(index => groups.flatMap(group => group[index] ? [group[index]!.id] : []));
    picked.push(...ordered.slice(0, count / 10));
  }
  return shuffle(picked, rng);
}
export function createMock(count: 20 | 60 | 100, seed: string, minutes: number, penalty: 0 | 0.2, now = Date.now(), id = `${now}-${seed}`): MockState {
  if (!Number.isInteger(minutes) || minutes < 1 || minutes > 240 || ![0, 0.2].includes(penalty)) throw new RangeError('Choose 1–240 whole minutes and a supported penalty.');
  return { version: 1, pending: null, session: { id, seed, kind: 'round', ids: selectMockQuestions(count, seed), started: now, deadline: now + minutes * 60_000, penalty, answers: {}, flagged: [], cursor: 0, finished: null, reason: null, recorded: false } };
}
/** Invalid structural data discards the session; unknown answers and flags are dropped. No saved score is trusted. */
export function parseMock(raw: string | null, now = Date.now()): MockState {
  const empty = emptyMock();
  if (!raw || raw.length > 300_000) return empty;
  try {
    const data: unknown = JSON.parse(raw);
    if (!object(data) || data.version !== 1) return empty;
    if (data.session === null) return empty;
    const s = data.session;
    if (!object(s) || typeof s.id !== 'string' || !s.id || s.id.length > 200 || typeof s.seed !== 'string' || !s.seed || s.seed.length > 100 || !['round', 'retry'].includes(String(s.kind)) || !Array.isArray(s.ids) || s.ids.length < 1 || s.ids.length > 100 || !s.ids.every(id => typeof id === 'string' && bank.has(id)) || new Set(s.ids).size !== s.ids.length || !time(s.started) || s.started > now || !time(s.deadline) || s.deadline <= s.started || s.deadline - s.started > 240 * 60_000 || ![0, 0.2].includes(Number(s.penalty)) || typeof s.penalty !== 'number') return empty;
    if (s.kind === 'round' && (![20, 60, 100].includes(s.ids.length) || JSON.stringify(s.ids) !== JSON.stringify(selectMockQuestions(s.ids.length as 20 | 60 | 100, s.seed)))) return empty;
    const finished = s.finished;
    if (finished !== null && (!time(finished) || finished < s.started || finished > Math.min(now, s.deadline))) return empty;
    if (finished === null ? s.reason !== null || s.recorded !== false : !['manual', 'expiry'].includes(String(s.reason)) || typeof s.recorded !== 'boolean' || (s.reason === 'expiry' && finished !== s.deadline)) return empty;
    const ids = s.ids as string[], answers: Record<string, number> = {};
    for (const id of ids) { const a = object(s.answers) ? s.answers[id] : undefined; if (typeof a === 'number' && Number.isInteger(a) && a >= 0 && a <= 3) answers[id] = a; }
    const session: MockSession = { id: s.id, seed: s.seed, kind: s.kind as MockSession['kind'], ids: [...ids], started: s.started, deadline: s.deadline, penalty: s.penalty as 0 | 0.2, answers, flagged: Array.isArray(s.flagged) ? [...new Set(s.flagged.filter((id): id is string => typeof id === 'string' && ids.includes(id)))] : [], cursor: typeof s.cursor === 'number' && Number.isInteger(s.cursor) ? Math.max(0, Math.min(ids.length - 1, s.cursor)) : 0, finished: finished as number | null, reason: s.reason as MockSession['reason'], recorded: s.recorded as boolean };
    const result: MockState = { version: 1, session, pending: null };
    if (data.pending !== null && data.pending !== undefined) {
      const p = data.pending;
      if (!object(p) || p.sessionId !== session.id || session.finished === null || session.recorded) return empty;
      const before = parseState(JSON.stringify(p.before)), after = applyAnswers(before, session);
      // Never allow an imported journal to inject unrelated history or change bookmarks.
      if (JSON.stringify(after) !== JSON.stringify(parseState(JSON.stringify(p.after)))) return empty;
      result.pending = { sessionId: session.id, before, after };
    }
    return result;
  } catch { return empty; }
}
export function exportMock(state: MockState, now = Date.now()): MockState { return parseMock(JSON.stringify(state), now); }
export function remainingMock(s: MockSession, now = Date.now()): number { return s.finished === null ? Math.max(0, s.deadline - now) : 0; }
export function answerMock(state: MockState, id: string, selected: number | null, now = Date.now()): MockState {
  const s = state.session;
  if (!s || s.finished !== null || now >= s.deadline || !s.ids.includes(id) || (selected !== null && (!Number.isInteger(selected) || selected < 0 || selected > 3))) return state;
  const answers = { ...s.answers }; if (selected === null) delete answers[id]; else answers[id] = selected;
  return { ...state, session: { ...s, answers } };
}
export function finaliseMock(state: MockState, reason: 'manual' | 'expiry', now = Date.now()): MockState {
  const s = state.session;
  if (!s || s.finished !== null || (reason === 'expiry' && now < s.deadline)) return state;
  return { ...state, session: { ...s, finished: Math.min(now, s.deadline), reason: now >= s.deadline ? 'expiry' : 'manual' } };
}
function applyAnswers(practice: PracticeState, s: MockSession): PracticeState {
  for (const id of s.ids) if (s.answers[id] !== undefined) practice = recordAnswer(practice, bank.get(id)!, s.answers[id]!, s.finished!);
  // Canonical question order matters when a shuffled multi-answer journal is compared
  // with history read back through parseState after an interrupted storage write.
  return parseState(JSON.stringify(practice));
}
/** Call under the same Web Lock across tabs. A failed write leaves a recoverable journal.
 * On unexpected external practice changes, stop rather than overwrite another tool's work. */
export function persistMockFinalisation(storage: MockStorage, state: MockState): MockState {
  const s = state.session;
  if (!s || s.finished === null || s.recorded) return state;
  const durable = parseMock(storage.getItem(MOCK_KEY));
  if (durable.session && durable.session.id !== s.id) throw new Error('Another round replaced this session. Reload to use the saved round.');
  if (durable.session?.id === s.id && durable.session.recorded) return durable;
  let next = durable.session?.id === s.id && durable.pending ? durable : state;
  if (!next.pending) {
    const before = parseState(storage.getItem(STORAGE_KEY));
    next = { ...next, pending: { sessionId: s.id, before, after: applyAnswers(before, s) } };
    storage.setItem(MOCK_KEY, JSON.stringify(next));
  }
  const pending = next.pending!, current = JSON.stringify(parseState(storage.getItem(STORAGE_KEY)));
  if (current === JSON.stringify(pending.before)) storage.setItem(STORAGE_KEY, JSON.stringify(pending.after));
  else if (current !== JSON.stringify(pending.after)) throw new Error('Practice history changed during interrupted saving. Keep this session and export a backup before recovery.');
  const done: MockState = { version: 1, session: { ...s, recorded: true }, pending: null };
  storage.setItem(MOCK_KEY, JSON.stringify(done));
  return done;
}
export function mockReport(s: MockSession) {
  const rows = s.ids.map(id => { const q = bank.get(id)!; return { question: q, selected: s.answers[id], outcome: s.answers[id] === undefined ? 'blank' : s.answers[id] === q.answer ? 'correct' : 'wrong' }; });
  const tally = (items: typeof rows) => { const correct = items.filter(r => r.outcome === 'correct').length, wrong = items.filter(r => r.outcome === 'wrong').length; return { correct, wrong, blank: items.length - correct - wrong, score: Math.round((correct - wrong * s.penalty) * 10) / 10 }; };
  return { ...tally(rows), rows, chapters: Array.from({ length: 10 }, (_, i) => ({ chapter: i + 1, ...tally(rows.filter(r => r.question.section.startsWith(`${i + 1}.`))) })) };
}
export function retryWrong(s: MockSession, seed: string, minutes: number, now = Date.now(), id = `${now}-${seed}`): MockState {
  if (s.finished === null) throw new Error('Finish the round first.');
  const ids = shuffle(mockReport(s).rows.filter(r => r.outcome === 'wrong').map(r => r.question.id), random(seed));
  if (!ids.length) throw new Error('No wrong answers to retry.');
  const state = createMock(20, seed, minutes, s.penalty, now, id);
  return { ...state, session: { ...state.session!, kind: 'retry', ids } };
}
