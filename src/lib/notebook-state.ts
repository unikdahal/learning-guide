import {pscWorkstreams} from '../data/psc-workstreams.ts';
import {practiceQuestions} from '../data/preparation-practice.ts';
import {validExamDate} from './exam-calendar.ts';
export const NOTEBOOK_KEY='exam-preparation:notebook:v1';
export type WorkStatus='unseen'|'learning'|'review'|'checked';
export type MistakeCause='concept'|'calculation'|'reading'|'time'|'uncertain';
export interface WorkRecord {status:WorkStatus; notes:string; sourceUrl:string; checkedDate:string; updated:number}
export interface MistakeNote {cause:MistakeCause; correction:string; nextAction:string; reviewed:boolean; updated:number}
export interface NotebookState {version:1; work:Record<string,WorkRecord>; mistakes:Record<string,MistakeNote>}
export const emptyNotebook=():NotebookState=>({version:1,work:{},mistakes:{}});
const statuses:WorkStatus[]=['unseen','learning','review','checked'];
const causes:MistakeCause[]=['concept','calculation','reading','time','uncertain'];
const object=(v:unknown):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v);
export function referenceUrl(value:unknown):string|null {
  if(value==='')return '';
  if(typeof value!=='string'||value.length>800)return null;
  try{const url=new URL(value);return ['https:','http:'].includes(url.protocol)&&!url.username&&!url.password&&url.href.length<=800?url.href:null;}catch{return null;}
}
const timestamp=(value:unknown,now:number):value is number=>typeof value==='number'&&Number.isSafeInteger(value)&&value>=0&&value<=now+86_400_000;
export function parseNotebook(raw:string|null,now=Date.now()):NotebookState {
  const state=emptyNotebook();if(!raw||raw.length>400_000)return state;
  try {
    const data:unknown=JSON.parse(raw);if(!object(data)||data.version!==1)return state;
    if(object(data.work))for(const item of pscWorkstreams) {
      const row=data.work[item.id];if(!object(row))continue;
      const url=referenceUrl(row.sourceUrl);
      if(!statuses.includes(row.status as WorkStatus)||typeof row.notes!=='string'||row.notes.length>600||url===null||!validExamDate(row.checkedDate)||!timestamp(row.updated,now))continue;
      state.work[item.id]={status:row.status as WorkStatus,notes:row.notes,sourceUrl:url,checkedDate:row.checkedDate,updated:Math.min(row.updated,now)};
    }
    if(object(data.mistakes))for(const question of practiceQuestions) {
      const row=data.mistakes[question.id];if(!object(row))continue;
      if(!causes.includes(row.cause as MistakeCause)||typeof row.correction!=='string'||row.correction.length>1200||typeof row.nextAction!=='string'||row.nextAction.length>400||typeof row.reviewed!=='boolean'||!timestamp(row.updated,now))continue;
      state.mistakes[question.id]={cause:row.cause as MistakeCause,correction:row.correction,nextAction:row.nextAction,reviewed:row.reviewed&&row.correction.trim().length>0,updated:Math.min(row.updated,now)};
    }
  }catch{/* Malformed browser data must not block study. */}
  return state;
}
export function noteCurrent(note:MistakeNote|undefined,lastAttempt=0):boolean {
  return !!note&&note.reviewed&&!!note.correction.trim()&&note.updated>=lastAttempt;
}
