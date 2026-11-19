import {pscWorkstreams} from '../../data/psc-workstreams.ts';
import {practiceQuestions} from '../../data/preparation-practice.ts';
import {studyTopics} from '../../data/study-map.ts';
import {STORAGE_KEY,parseState} from '../preparation/state.ts';
import {NOTEBOOK_KEY,emptyNotebook,parseNotebook,referenceUrl,noteCurrent,type NotebookState,type WorkStatus,type MistakeCause} from '../../lib/notebook-state.ts';
import {validExamDate} from '../../lib/exam-calendar.ts';

class PreparationNotebook extends HTMLElement {
  private state=emptyNotebook();
  private events?:AbortController;
  private memoryOnly=false;
  private questionId='';
  private quizAvailable=true;
  private $<T extends HTMLElement=HTMLElement>(selector:string):T {const el=this.querySelector<T>(selector);if(!el)throw new Error(`Missing notebook control ${selector}`);return el;}
  private href(path:string){return `${this.dataset.base}/${path}`;}
  private say(message:string){const el=this.$('[data-notebook-status]');if(el.textContent!==message)el.textContent=message;}
  private read():NotebookState {try{return parseNotebook(localStorage.getItem(NOTEBOOK_KEY));}catch{this.memoryOnly=true;return this.state;}}
  private change(update:(state:NotebookState)=>NotebookState){
    this.state=update(this.memoryOnly?this.state:this.read());
    try {if(this.memoryOnly)throw new Error('unavailable');localStorage.setItem(NOTEBOOK_KEY,JSON.stringify(this.state));this.say('Notebook saved in this browser. Use the revision studio for a complete preparation backup.');}
    catch{this.memoryOnly=true;this.say('Browser storage is unavailable. Notes work during this visit; copy important text before leaving.');}
  }
  private practice(){try{this.quizAvailable=true;return parseState(localStorage.getItem(STORAGE_KEY));}catch{this.quizAvailable=false;return parseState(null);}}
  connectedCallback(){
    if(this.events)return;this.events=new AbortController();const signal=this.events.signal;this.state=this.read();
    this.querySelectorAll<HTMLElement>('[data-notebook-live]').forEach(el=>el.hidden=false);
    for(const selector of ['[data-work-target]','[data-work-kind]','[data-work-status]','[data-work-search]'])this.$(selector).addEventListener('input',()=>this.renderWork(),{signal});
    const now=new Date();const today=`${now.getFullYear()}-${String(now.getMonth()+1).padStart(2,'0')}-${String(now.getDate()).padStart(2,'0')}`;
    for(const row of this.querySelectorAll<HTMLElement>('[data-work]')){
      row.querySelector<HTMLInputElement>('[data-work-date]')!.max=today;
      for(const control of row.querySelectorAll('input,textarea,select'))control.addEventListener('change',()=>this.saveWork(row),{signal});
      row.querySelector('[data-work-notes]')!.addEventListener('input',()=>this.saveWork(row),{signal});
    }
    for(const selector of ['[data-mistake-view]','[data-mistake-search]'])this.$(selector).addEventListener('input',()=>{this.questionId='';this.renderMistakes();this.renderEditor();},{signal});
    this.$('[data-mistake-select]').addEventListener('change',()=>{this.questionId=this.$<HTMLSelectElement>('[data-mistake-select]').value;this.renderEditor();},{signal});
    for(const selector of ['[data-note-correction]','[data-note-next]'])this.$(selector).addEventListener('input',()=>this.saveNote(),{signal});
    for(const selector of ['[data-note-cause]','[data-note-reviewed]'])this.$(selector).addEventListener('change',()=>this.saveNote(),{signal});
    this.$('[data-note-delete]').addEventListener('click',()=>{
      if(!this.questionId||!confirm('Delete the personal note for this question? The actual quiz history is kept.'))return;
      const id=this.questionId;this.change(state=>{const mistakes={...state.mistakes};delete mistakes[id];return {...state,mistakes};});this.renderMistakes();this.renderEditor();
    },{signal});
    window.addEventListener('storage',event=>{
      if(event.key===NOTEBOOK_KEY||event.key===null){this.state=this.read();this.renderAll();}
      else if(event.key===STORAGE_KEY){this.renderMistakes();this.renderEditor();}
    },{signal});
    this.renderAll();if(this.memoryOnly)this.say('Browser storage is unavailable. Notes work during this visit; copy important text before leaving.');
    else this.say('Personal statuses, notes and references are stored in this browser. They are self-entered, not verified exam results.');
  }
  disconnectedCallback(){this.events?.abort();this.events=undefined;}
  private renderAll(){
    for(const row of this.querySelectorAll<HTMLElement>('[data-work]')){
      const record=this.state.work[row.dataset.work!];
      row.querySelector<HTMLSelectElement>('[data-work-state]')!.value=record?.status??'unseen';
      row.querySelector<HTMLTextAreaElement>('[data-work-notes]')!.value=record?.notes??'';
      row.querySelector<HTMLInputElement>('[data-work-url]')!.value=record?.sourceUrl??'';
      row.querySelector<HTMLInputElement>('[data-work-date]')!.value=record?.checkedDate??'';
      this.referenceLink(row);
    }
    this.renderWork();this.renderMistakes();this.renderEditor();
  }
  private saveWork(row:HTMLElement){
    const item=pscWorkstreams.find(item=>item.id===row.dataset.work)!;
    const urlInput=row.querySelector<HTMLInputElement>('[data-work-url]')!,dateInput=row.querySelector<HTMLInputElement>('[data-work-date]')!;
    const url=referenceUrl(urlInput.value.trim()),date=dateInput.value;
    if(url===null||!validExamDate(date)||!dateInput.validity.valid){this.say(`The reference for ${item.title} was not saved. Use an http/https URL without credentials and a valid check date no later than today. Your typed values remain visible.`);return;}
    this.change(state=>({...state,work:{...state.work,[item.id]:{status:row.querySelector<HTMLSelectElement>('[data-work-state]')!.value as WorkStatus,notes:row.querySelector<HTMLTextAreaElement>('[data-work-notes]')!.value.slice(0,600),sourceUrl:url,checkedDate:date,updated:Date.now()}}}));
    this.referenceLink(row);this.renderWork();
  }
  private referenceLink(row:HTMLElement){
    const url=this.state.work[row.dataset.work!]?.sourceUrl??'';const link=row.querySelector<HTMLAnchorElement>('[data-work-reference]')!;
    link.hidden=!url;if(url)link.href=url;else link.removeAttribute('href');
  }
  private renderWork(){
    const target=this.$<HTMLSelectElement>('[data-work-target]').value,kind=this.$<HTMLSelectElement>('[data-work-kind]').value,status=this.$<HTMLSelectElement>('[data-work-status]').value,query=this.$<HTMLInputElement>('[data-work-search]').value.toLowerCase().trim();
    let visible=0,checked=0;
    for(const row of this.querySelectorAll<HTMLElement>('[data-work]')){
      const item=pscWorkstreams.find(item=>item.id===row.dataset.work)!;const record=this.state.work[item.id];const value=record?.status??'unseen';
      row.hidden=!((target==='all'||item.targets.includes(target as 'ce'|'co'))&&(kind==='all'||item.kind===kind)&&(status==='all'||(status==='checked'?value==='checked':value!=='checked'))&&`${item.title} ${item.scope} ${item.output}`.toLowerCase().includes(query));
      row.querySelector('[data-work-badge]')!.textContent=({unseen:'Not started',learning:'Learning',review:'Needs review',checked:'Self-checked'})[value];
      if(!row.hidden){visible++;if(value==='checked')checked++;}
    }
    this.$('[data-work-summary]').textContent=`${visible} workstreams in this view · ${checked} self-checked. Counts are not exam weighting or a readiness estimate.`;
    const progress=this.$<HTMLProgressElement>('[data-work-progress]');progress.max=Math.max(1,visible);progress.value=checked;
    this.$('[data-work-empty]').hidden=visible!==0;
  }
  private questions(){
    const practice=this.practice(),view=this.$<HTMLSelectElement>('[data-mistake-view]').value,query=this.$<HTMLInputElement>('[data-mistake-search]').value.trim().toLowerCase();
    return practiceQuestions.filter(q=>(view==='all'||(view==='noted'?!!this.state.mistakes[q.id]:!!practice.answers[q.id]&&practice.answers[q.id]!.selected!==q.answer))&&`${q.prompt} NEC ${q.section}`.toLowerCase().includes(query));
  }
  private renderMistakes(){
    const questions=this.questions(),select=this.$<HTMLSelectElement>('[data-mistake-select]');select.replaceChildren();
    for(const q of questions){const option=document.createElement('option');option.value=q.id;option.textContent=`NEC ${q.section} · ${q.prompt}`;select.append(option);}
    if(!questions.some(q=>q.id===this.questionId))this.questionId=questions[0]?.id??'';
    select.value=this.questionId;select.disabled=questions.length===0;
    this.$('[data-mistake-summary]').textContent=this.quizAvailable?`${questions.length} matching questions. Notes never change correct/wrong counts.`:'Quiz history is unavailable. Choose “Any original question” to keep a note during this visit.';
    this.$('[data-mistake-empty]').hidden=questions.length!==0;this.$('[data-note-editor]').hidden=questions.length===0;
  }
  private renderEditor(){
    const q=practiceQuestions.find(q=>q.id===this.questionId);if(!q)return;
    const note=this.state.mistakes[q.id],record=this.practice().answers[q.id];
    this.$('[data-note-scope]').textContent=`${q.id} · NEC ${q.section} · Original question`;
    this.$('[data-note-question]').textContent=q.prompt;
    this.$('[data-note-attempt]').textContent=record?`Most recent submitted answer: ${q.options[record.selected]}. ${record.selected===q.answer?'Correct':'Incorrect'}; ${record.correct}/${record.attempts} correct across submissions.`:'No submitted answer is recorded here. You may still record uncertainty or a lucky guess.';
    this.$('[data-note-answer]').textContent=`Correct answer: ${q.options[q.answer]}`;this.$('[data-note-explanation]').textContent=q.explanation;
    this.$<HTMLDetailsElement>('[data-note-solution]').open=false;
    this.$<HTMLSelectElement>('[data-note-cause]').value=note?.cause??'concept';
    this.$<HTMLTextAreaElement>('[data-note-correction]').value=note?.correction??'';
    this.$<HTMLTextAreaElement>('[data-note-next]').value=note?.nextAction??'';
    this.$<HTMLInputElement>('[data-note-reviewed]').checked=note?.reviewed??false;
    this.updateNoteStatus();
    const topic=studyTopics.find(t=>t.id===q.section);if(topic)this.$<HTMLAnchorElement>('[data-note-lesson]').href=this.href(topic.lesson);
  }
  private saveNote(){
    if(!this.questionId)return;const id=this.questionId,correction=this.$<HTMLTextAreaElement>('[data-note-correction]').value.slice(0,1200),nextAction=this.$<HTMLTextAreaElement>('[data-note-next]').value.slice(0,400),reviewed=this.$<HTMLInputElement>('[data-note-reviewed]').checked&&!!correction.trim(),cause=this.$<HTMLSelectElement>('[data-note-cause]').value as MistakeCause;
    this.change(state=>({...state,mistakes:{...state.mistakes,[id]:{cause,correction,nextAction,reviewed,updated:Date.now()}}}));
    this.updateNoteStatus();this.renderMistakes();
  }
  private updateNoteStatus(){
    const note=this.state.mistakes[this.questionId],record=this.practice().answers[this.questionId];
    const checkbox=this.$<HTMLInputElement>('[data-note-reviewed]');checkbox.disabled=!note?.correction.trim();checkbox.checked=noteCurrent(note,record?.updatedAt);
    this.$('[data-note-status]').textContent=noteCurrent(note,record?.updatedAt)?'Your reasoning is marked checked for the latest recorded attempt. Retry a fresh problem to test transfer.':note?.reviewed?'A newer quiz attempt exists after this note. Re-check the reasoning and update it.':'Write a correction, then check it against the explanation. No self-review has been recorded for the current attempt.';
  }
}
if(!customElements.get('preparation-notebook'))customElements.define('preparation-notebook',PreparationNotebook);
