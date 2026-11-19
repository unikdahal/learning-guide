import {validExamDate} from './exam-calendar.ts';
export {validExamDate,daysUntil} from './exam-calendar.ts';
import { studyTopics } from '../data/study-map.ts';
import { writtenPrompts } from '../data/written-practice.ts';
import { practiceQuestions } from '../data/preparation-practice.ts';
import { parseStudy, type StudyState, DAY } from './study-state.ts';
import { sanitizeState, parseState, type PracticeState } from '../components/preparation/state.ts';
import {parseNotebook,type NotebookState} from './notebook-state.ts';
import {parseMock,type MockState} from './mock-state.ts';

export const REVISION_KEY = 'exam-preparation:revision:v1';
export type PreparationTarget = 'combined' | 'nec' | 'ce' | 'co';
export type RecallGrade = 'again' | 'building' | 'ready';
export interface RecallRecord { grade: RecallGrade; reviewed: number; reviews: number }
export interface Draft { text: string; checks: boolean[]; updated: number }
export interface RevisionState {
  version: 1;
  settings: {target: PreparationTarget; minutes: number; examDate: string};
  recall: Record<string, RecallRecord>;
  drafts: Record<string, Draft>;
}
export const emptyRevision = (): RevisionState => ({version:1,settings:{target:'combined',minutes:90,examDate:''},recall:{},drafts:{}});
const object = (value: unknown): value is Record<string, unknown> => !!value && typeof value === 'object' && !Array.isArray(value);
const targets: PreparationTarget[] = ['combined','nec','ce','co'];
const grades: RecallGrade[] = ['again','building','ready'];
export const recallIntervals: Record<RecallGrade, number> = {again:1,building:3,ready:7};

export function parseRevision(raw: string | null, now = Date.now()): RevisionState {
  const state=emptyRevision();
  if(!raw || raw.length>300_000) return state;
  try {
    const data: unknown=JSON.parse(raw);
    if(!object(data)||data.version!==1) return state;
    if(object(data.settings)) {
      const {target,minutes,examDate}=data.settings;
      if(targets.includes(target as PreparationTarget)) state.settings.target=target as PreparationTarget;
      if(typeof minutes==='number'&&Number.isInteger(minutes)&&minutes>=20&&minutes<=180) state.settings.minutes=minutes;
      if(validExamDate(examDate)) state.settings.examDate=examDate;
    }
    if(object(data.recall)) for(const topic of studyTopics) {
      const row=data.recall[topic.id];
      if(!object(row)||!grades.includes(row.grade as RecallGrade)||typeof row.reviewed!=='number'||!Number.isSafeInteger(row.reviewed)||row.reviewed<0||row.reviewed>now+DAY||typeof row.reviews!=='number'||!Number.isSafeInteger(row.reviews)||row.reviews<1||row.reviews>1_000_000)continue;
      state.recall[topic.id]={grade:row.grade as RecallGrade,reviewed:Math.min(row.reviewed,now),reviews:row.reviews};
    }
    if(object(data.drafts)) for(const prompt of writtenPrompts) {
      const draft=data.drafts[prompt.id];
      if(!object(draft)||typeof draft.text!=='string'||draft.text.length>8000||!Array.isArray(draft.checks)||draft.checks.length!==prompt.checks.length||!draft.checks.every(v=>typeof v==='boolean')||typeof draft.updated!=='number'||!Number.isSafeInteger(draft.updated)||draft.updated<0||draft.updated>now+DAY)continue;
      state.drafts[prompt.id]={text:draft.text,checks:draft.checks,updated:Math.min(draft.updated,now)};
    }
  }catch { /* Keep every tool usable with broken local data. */ }
  return state;
}
export function recallDue(record: RecallRecord | undefined, now=Date.now()): boolean {
  return !record || now-record.reviewed>=recallIntervals[record.grade]*DAY;
}
export function targetIncludes(section: string, target: PreparationTarget): boolean {
  const topic=studyTopics.find(t=>t.id===section);
  return !!topic && (target==='ce'?topic.ce!==null:target==='co'?topic.co!==null:true);
}
export const recallCards = studyTopics.flatMap(topic => {
  const question=practiceQuestions.find(q=>q.section===topic.id);
  return question ? [{section:topic.id,title:topic.title,lesson:topic.lesson,question}] : [];
});
export function sessionBlocks(minutes: number, target: PreparationTarget) {
  if(!Number.isInteger(minutes)||minutes<20||minutes>180)throw new RangeError('Choose 20–180 whole minutes.');
  if(minutes<40) return [{label:'Closed-book recall',minutes:5},{label:'One focused topic and a fresh question',minutes:minutes-5}];
  const retrieve=Math.round(minutes*.15),learn=Math.round(minutes*.35),practice=Math.round(minutes*.25);
  return [{label:'Closed-book recall',minutes:retrieve},{label:'Learn your priority topic',minutes:learn},{label:'Solve fresh questions',minutes:practice},{label:target==='nec'?'Explain, draw or trace from memory':'Write and check one answer',minutes:minutes-retrieve-learn-practice}];
}
export interface PreparationBackup {
  format: 'nec-loksewa-preparation'; version: 1; exportedAt: string;
  coverage: StudyState; practice: PracticeState; revision: RevisionState;
  notebook?: NotebookState;
  mock?: MockState;
}
export const MAX_PREPARATION_BACKUP_BYTES=2_000_000;
/** Validate the whole envelope before touching any browser storage. Each namespace sanitises its records. */
export function parsePreparationBackup(raw: string, now=Date.now()): PreparationBackup {
  if(raw.length>MAX_PREPARATION_BACKUP_BYTES)throw new Error('Backup exceeds the 2 MB limit.');
  const data: unknown=JSON.parse(raw);
  if(!object(data)||data.format!=='nec-loksewa-preparation'||data.version!==1||typeof data.exportedAt!=='string'||!Number.isFinite(Date.parse(data.exportedAt)))throw new Error('Not a preparation backup.');
  if(!object(data.coverage)||data.coverage.version!==1||!object(data.coverage.topics)||!object(data.practice)||data.practice.version!==1||!object(data.practice.answers)||!Array.isArray(data.practice.bookmarks)||!object(data.revision)||data.revision.version!==1||!object(data.revision.settings)||!object(data.revision.recall)||!object(data.revision.drafts))throw new Error('A preparation section is missing or invalid.');
  if(Object.hasOwn(data,'notebook')&&(!object(data.notebook)||data.notebook.version!==1||!object(data.notebook.work)||!object(data.notebook.mistakes)))throw new Error('Notebook section is invalid.');
  if(Object.hasOwn(data,'mock')&&(!object(data.mock)||data.mock.version!==1||!Object.hasOwn(data.mock,'session')||!Object.hasOwn(data.mock,'pending')))throw new Error('Mock section is invalid.');
  const practice=sanitizeState(data.practice);
  const mock=Object.hasOwn(data,'mock')?parseMock(JSON.stringify(data.mock),now):undefined;
  return {format:'nec-loksewa-preparation',version:1,exportedAt:data.exportedAt,
    coverage:parseStudy(JSON.stringify(data.coverage),studyTopics.map(t=>t.id),now),
    practice,revision:parseRevision(JSON.stringify(data.revision),now),
    ...(Object.hasOwn(data,'notebook')?{notebook:parseNotebook(JSON.stringify(data.notebook),now)}:{}),
    ...(mock?{mock}:{})};
}

/** Conflicting history is preserved in a backup; the mock tool must not replay it automatically. */
export function mockNeedsRecovery(backup: Pick<PreparationBackup,'mock'|'practice'>):boolean {
  if(!backup.mock?.pending)return false;
  const current=JSON.stringify(backup.practice),before=JSON.stringify(parseState(JSON.stringify(backup.mock.pending.before))),after=JSON.stringify(parseState(JSON.stringify(backup.mock.pending.after)));
  return current!==before&&current!==after;
}

interface PreparationStorage { getItem(key:string):string|null; setItem(key:string,value:string):void; removeItem(key:string):void }
/** Best-effort rollback when browser quota or permissions fail partway through a multi-store import. */
export function writePreparationStores(storage: PreparationStorage, entries: readonly (readonly [string,string])[]) {
  const previous=entries.map(([key])=>[key,storage.getItem(key)] as const);
  try {for(const [key,value] of entries)storage.setItem(key,value);}
  catch {
    let restored=true;
    for(const [key,value] of previous)try{if(value===null)storage.removeItem(key);else storage.setItem(key,value);}catch{restored=false;}
    throw new Error(restored?'Import failed; original preparation stores restored.':'Import failed and browser storage blocked full restoration. Keep the previous backup.');
  }
}
