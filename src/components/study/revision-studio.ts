import {withPreparationLock} from '../../lib/preparation-lock.ts';
import {studyTopics} from '../../data/study-map.ts';
import {writtenPrompts} from '../../data/written-practice.ts';
import {practiceQuestions} from '../../data/preparation-practice.ts';
import {STUDY_KEY,parseStudy,priority} from '../../lib/study-state.ts';
import {STORAGE_KEY,parseState} from '../preparation/state.ts';
import {REVISION_KEY,emptyRevision,parseRevision,recallCards,recallDue,sessionBlocks,daysUntil,validExamDate,parsePreparationBackup,writePreparationStores,mockNeedsRecovery,MAX_PREPARATION_BACKUP_BYTES,type RevisionState,type PreparationTarget,type RecallGrade,type Draft} from '../../lib/revision-state.ts';
import {NOTEBOOK_KEY,parseNotebook} from '../../lib/notebook-state.ts';
import {MOCK_KEY,parseMock,emptyMock} from '../../lib/mock-state.ts';

class RevisionStudio extends HTMLElement {
  private state=emptyRevision();
  private events?: AbortController;
  private memoryOnly=false;
  private cardIndex=0;
  private promptId=writtenPrompts[0]!.id;
  private timer?: ReturnType<typeof setInterval>;
  private deadline=0;
  private remaining=0;
  private $<T extends HTMLElement=HTMLElement>(selector:string): T {
    const element=this.querySelector<T>(selector);
    if(!element)throw new Error(`Missing revision control ${selector}`);
    return element;
  }
  private say(message:string) { if(this.$('[data-notice]').textContent!==message)this.$('[data-notice]').textContent=message; }
  private href(path:string) {return `${this.dataset.base}/${path}`;}
  private read(): RevisionState {
    try {return parseRevision(localStorage.getItem(REVISION_KEY));}
    catch {this.memoryOnly=true;return this.state;}
  }
  private change(update:(state:RevisionState)=>RevisionState) {
    this.state=update(this.memoryOnly?this.state:this.read());
    try {if(this.memoryOnly)throw new Error('Storage unavailable');localStorage.setItem(REVISION_KEY,JSON.stringify(this.state));this.say('Saved in this browser. Export a preparation backup before changing devices.');}
    catch {this.memoryOnly=true;this.say('Browser storage is unavailable. This visit still works; copy your draft before leaving. A complete backup needs readable browser storage.');}
  }
  connectedCallback() {
    if(this.events)return;
    this.events=new AbortController();const signal=this.events.signal;
    this.state=this.read();
    const listen=(selector:string,type:string,handler:EventListener)=>this.$(selector).addEventListener(type,handler,{signal});
    listen('[data-target]','change',()=>{
      const target=this.$<HTMLSelectElement>('[data-target]').value as PreparationTarget;
      this.change(state=>({...state,settings:{...state.settings,target}}));this.cardIndex=0;this.renderAll();
    });
    const settings=()=>{
      const input=this.$<HTMLInputElement>('[data-minutes]');const minutes=Number(input.value);
      const date=this.$<HTMLInputElement>('[data-exam-date]').value;
      if(!input.value || !Number.isInteger(minutes)||minutes<20||minutes>180||!validExamDate(date)) {this.say('Choose 20–180 whole minutes and a valid optional date. Your saved plan is unchanged.');return;}
      this.change(state=>({...state,settings:{...state.settings,minutes,examDate:date}}));this.renderPlan();
    };
    listen('[data-minutes]','change',settings);listen('[data-exam-date]','change',settings);
    for(const selector of ['[data-card-chapter]','[data-card-mode]'])listen(selector,'change',()=>{this.cardIndex=0;this.renderCard();});
    listen('[data-reveal]','click',()=>{this.$('[data-recall-answer]').hidden=false;this.$('[data-grades]').hidden=false;this.$('[data-reveal]').hidden=true;});
    listen('[data-skip]','click',()=>{this.cardIndex++;this.renderCard(true);});
    for(const button of this.querySelectorAll<HTMLButtonElement>('[data-grade]'))button.addEventListener('click',()=>{
      const card=this.cards()[this.cardIndex%this.cards().length];if(!card)return;
      const grade=button.dataset.grade as RecallGrade;
      this.change(state=>({...state,recall:{...state.recall,[card.section]:{grade,reviewed:Date.now(),reviews:Math.min(1_000_000,(state.recall[card.section]?.reviews??0)+1)}}}));
      // Keep the next due card in place when removal shortens the deck; advance in the full deck.
      if(this.$<HTMLSelectElement>('[data-card-mode]').value==='all')this.cardIndex++;
      this.renderCard(true);this.renderOverview();
    },{signal});
    listen('[data-prompt-select]','change',()=>{this.promptId=this.$<HTMLSelectElement>('[data-prompt-select]').value;this.renderWriting();});
    listen('[data-draft]','input',()=>this.saveDraft());
    listen('[data-timer-start]','click',()=>this.startTimer());
    listen('[data-timer-pause]','click',()=>this.pauseTimer());
    listen('[data-clear-draft]','click',()=>{
      if(!confirm('Clear this prompt’s draft and self-checks? Other drafts remain saved.'))return;
      this.change(state=>{const drafts={...state.drafts};delete drafts[this.promptId];return {...state,drafts};});this.renderWriting();this.renderOverview();
    });
    listen('[data-backup-export]','click',()=>this.exportBackup());
    listen('[data-backup-import]','change',event=>{void this.importBackup(event.target as HTMLInputElement);});
    window.addEventListener('storage',event=>{
      if(event.key===REVISION_KEY || event.key===null){this.state=this.read();this.renderAll();}
      else if(event.key===STUDY_KEY || event.key===STORAGE_KEY){this.renderOverview();this.renderPlan();}
    },{signal});
    this.$('[data-live]').hidden=false;this.$('[data-static]').hidden=true;
    this.renderAll();
    if(this.memoryOnly)this.say('Browser storage is unavailable. You can practise during this visit; copy drafts before leaving.');
  }
  disconnectedCallback(){this.stopTimer();this.events?.abort();this.events=undefined;}
  private includes(section:string) {
    const topic=studyTopics.find(t=>t.id===section);const target=this.state.settings.target;
    return !!topic && (target==='ce'?topic.ce!==null:target==='co'?topic.co!==null:true);
  }
  private cards() {
    const chapter=this.$<HTMLSelectElement>('[data-card-chapter]').value;
    const dueOnly=this.$<HTMLSelectElement>('[data-card-mode]').value==='due';
    return recallCards.filter(card=>this.includes(card.section)&&(chapter==='all'||card.section.startsWith(`${chapter}.`))&&(!dueOnly||recallDue(this.state.recall[card.section])))
      .sort((a,b)=>(this.state.recall[a.section]?.reviewed??Number.MAX_SAFE_INTEGER)-(this.state.recall[b.section]?.reviewed??Number.MAX_SAFE_INTEGER));
  }
  private renderAll(){
    this.$<HTMLSelectElement>('[data-target]').value=this.state.settings.target;
    this.$<HTMLInputElement>('[data-minutes]').value=String(this.state.settings.minutes);
    this.$<HTMLInputElement>('[data-exam-date]').value=this.state.settings.examDate;
    this.renderOverview();this.renderPlan();this.renderCard();this.renderPromptOptions();this.renderWriting();
  }
  private renderOverview(){
    let coverage=undefined,practice=undefined,readable=true;
    try {coverage=parseStudy(localStorage.getItem(STUDY_KEY),studyTopics.map(t=>t.id));practice=parseState(localStorage.getItem(STORAGE_KEY));}catch{readable=false;}
    const topics=studyTopics.filter(t=>this.includes(t.id));
    const ready=topics.filter(t=>coverage?.topics[t.id]?.confidence==='ready').length;
    const wrong=practiceQuestions.filter(q=>this.includes(q.section)&&practice?.answers[q.id]&&practice.answers[q.id]!.selected!==q.answer).length;
    const due=recallCards.filter(c=>this.includes(c.section)&&recallDue(this.state.recall[c.section])).length;
    const drafts=Object.values(this.state.drafts).filter(d=>d.text.trim().length>0).length;
    const metrics=this.$('[data-metrics]');metrics.replaceChildren();
    for(const [value,label] of [[readable?`${ready}/${topics.length}`:'Unavailable','self-assessed recall ready'],[readable?String(wrong):'Unavailable','latest quiz mistakes'],[String(due),'unseen or due recall cards'],[String(drafts),'nonempty written drafts']]){
      const p=document.createElement('p'),strong=document.createElement('strong'),span=document.createElement('span');strong.textContent=value!;span.textContent=label!;p.append(strong,span);metrics.append(p);
    }
  }
  private renderPlan(){
    const {minutes,target,examDate}=this.state.settings;
    const list=this.$('[data-blocks]');list.replaceChildren();
    for(const block of sessionBlocks(minutes,target)) {const item=document.createElement('li'),time=document.createElement('strong');time.textContent=`${block.minutes} min · `;item.append(time,document.createTextNode(block.label));list.append(item);}
    const days=daysUntil(examDate);
    this.$('[data-date-note]').textContent=days===null?'Set your own confirmed exam date if useful; the studio does not infer an official schedule.':days<0?'That date is in the past. Update it to your next confirmed exam date.':days===0?'Your selected exam date is today. Use a light review plan and follow your admit-card instructions.':`${days} calendar days until your selected date. This is your entered date, not an official exam notice.`;
    let coverage;try{coverage=parseStudy(localStorage.getItem(STUDY_KEY),studyTopics.map(t=>t.id));}catch{coverage=parseStudy(null,[]);}
    const next=studyTopics.filter(t=>this.includes(t.id)).sort((a,b)=>priority(coverage.topics[b.id],b.ce!==null)-priority(coverage.topics[a.id],a.ce!==null))[0];
    const root=this.$('[data-priority]');root.replaceChildren();
    if(next) {const lead=document.createElement('p');lead.textContent=`Suggested focus: NEC ${next.id} · ${next.title}`;const link=document.createElement('a');link.href=this.href(next.lesson);link.textContent='Open the lesson';root.append(lead,link);}
  }
  private renderCard(focus=false){
    const deck=this.cards();if(this.cardIndex>=deck.length)this.cardIndex=0;
    const card=deck[this.cardIndex];
    this.$('[data-card]').hidden=!card;this.$('[data-card-empty]').hidden=!!card;
    this.$('[data-card-count]').textContent=`${deck.length} cards in this view. Recall intervals are a simple 1/3/7-day study rule, not an adaptive mastery model.`;
    if(!card)return;
    this.$('[data-card-tag]').textContent=`NEC ${card.section} · ${card.title} · Original recall prompt`;
    this.$('[data-card-prompt]').textContent=card.question.prompt;
    this.$('[data-card-answer]').textContent=card.question.options[card.question.answer]!;
    this.$('[data-card-explanation]').textContent=card.question.explanation;
    this.$<HTMLAnchorElement>('[data-card-lesson]').href=this.href(card.lesson);
    this.$('[data-recall-answer]').hidden=true;this.$('[data-grades]').hidden=true;this.$('[data-reveal]').hidden=false;
    if(focus)this.$('[data-card-prompt]').focus();
  }
  private renderPromptOptions(){
    const target=this.state.settings.target;
    const prompts=writtenPrompts.filter(p=>target==='ce'?p.targets.includes('ce'):target==='co'?p.targets.includes('co'):target==='nec'?p.scope.includes('NEC'):true);
    const select=this.$<HTMLSelectElement>('[data-prompt-select]');select.replaceChildren();
    for(const prompt of prompts){const option=document.createElement('option');option.value=prompt.id;option.textContent=`${prompt.title} · ${prompt.marks}-mark practice`;select.append(option);}
    if(!prompts.some(p=>p.id===this.promptId))this.promptId=prompts[0]!.id;
    select.value=this.promptId;
  }
  private prompt(){return writtenPrompts.find(p=>p.id===this.promptId)!;}
  private draft():Draft {return this.state.drafts[this.promptId]??{text:'',checks:this.prompt().checks.map(()=>false),updated:0};}
  private renderWriting(){
    this.stopTimer();this.remaining=this.prompt().minutes*60;
    const prompt=this.prompt(),draft=this.draft();
    this.$('[data-writing-tag]').textContent=`${prompt.scope} · Original ${prompt.marks}-mark exercise`;
    this.$('[data-writing-question]').textContent=prompt.question;
    this.$<HTMLTextAreaElement>('[data-draft]').value=draft.text;
    this.$('[data-writing-outline]').textContent=prompt.outline;
    this.$<HTMLDetailsElement>('[data-writing-guide]').open=false;
    const checks=this.$('[data-checks]');checks.replaceChildren();
    for(const [index,text] of prompt.checks.entries()) {
      const label=document.createElement('label'),input=document.createElement('input'),span=document.createElement('span');input.type='checkbox';input.checked=draft.checks[index]??false;span.textContent=text;
      input.addEventListener('change',()=>this.saveDraft());label.append(input,span);checks.append(label);
    }
    this.$('[data-writing-clock]').textContent=`${prompt.minutes}:00 suggested`;
    this.$('[data-timer-start]').textContent=`Start ${prompt.minutes}-minute practice`;
    this.$<HTMLButtonElement>('[data-timer-pause]').disabled=true;
    this.renderDraftCounts();
  }
  private saveDraft(){
    const text=this.$<HTMLTextAreaElement>('[data-draft]').value.slice(0,8000);
    const checks=Array.from(this.querySelectorAll<HTMLInputElement>('[data-checks] input')).map(i=>i.checked);
    this.change(state=>({...state,drafts:{...state.drafts,[this.promptId]:{text,checks,updated:Date.now()}}}));
    this.renderDraftCounts();this.renderOverview();
  }
  private renderDraftCounts(){
    const draft=this.draft();const words=draft.text.trim()?draft.text.trim().split(/\s+/).length:0;
    this.$('[data-word-count]').textContent=`${words} whitespace-separated words · ${draft.text.length} / 8000 characters`;
    this.$('[data-writing-progress]').textContent=`${draft.checks.filter(Boolean).length} / ${this.prompt().checks.length} self-checks checked. Checklist confidence is changed separately on the preparation desk.`;
  }
  private stopTimer(){if(this.timer)clearInterval(this.timer);this.timer=undefined;this.deadline=0;}
  private startTimer(){
    if(this.timer)return;if(this.remaining<=0)this.remaining=this.prompt().minutes*60;
    this.deadline=Date.now()+this.remaining*1000;this.$<HTMLButtonElement>('[data-timer-pause]').disabled=false;
    const tick=()=>{
      this.remaining=Math.max(0,Math.ceil((this.deadline-Date.now())/1000));
      this.$('[data-writing-clock]').textContent=`${Math.floor(this.remaining/60)}:${String(this.remaining%60).padStart(2,'0')} remaining`;
      if(this.remaining===0){this.stopTimer();this.$<HTMLButtonElement>('[data-timer-pause]').disabled=true;this.$('[data-timer-start]').textContent='Restart practice timer';this.say('Writing time ended. Your draft is still editable; compare it with the outline when ready.');}
    };
    tick();this.timer=setInterval(tick,1000);
  }
  private pauseTimer(){
    if(!this.timer)return;this.remaining=Math.max(0,Math.ceil((this.deadline-Date.now())/1000));this.stopTimer();
    this.$('[data-timer-start]').textContent='Resume practice timer';this.$<HTMLButtonElement>('[data-timer-pause]').disabled=true;
    this.$('[data-writing-clock]').textContent=`Paused · ${Math.floor(this.remaining/60)}:${String(this.remaining%60).padStart(2,'0')}`;
  }
  private exportBackup(){
    try {
      const backup={format:'nec-loksewa-preparation',version:1,exportedAt:new Date().toISOString(),
        coverage:parseStudy(localStorage.getItem(STUDY_KEY),studyTopics.map(t=>t.id)),practice:parseState(localStorage.getItem(STORAGE_KEY)),revision:parseRevision(localStorage.getItem(REVISION_KEY)),notebook:parseNotebook(localStorage.getItem(NOTEBOOK_KEY)),mock:parseMock(localStorage.getItem(MOCK_KEY))};
      parsePreparationBackup(JSON.stringify(backup));
      const blob=new Blob([JSON.stringify(backup,null,2)],{type:'application/json'});if(blob.size>MAX_PREPARATION_BACKUP_BYTES)throw new Error('Backup too large');
      const href=URL.createObjectURL(blob),link=document.createElement('a');
      link.href=href;link.download='nec-loksewa-preparation.json';link.click();setTimeout(()=>URL.revokeObjectURL(href),1000);
      this.say(`Preparation backup exported. It includes notes and drafts in plain text; keep the file where you intend.${mockNeedsRecovery(backup)?' The mock has a conflicting recovery journal; both it and the current answer history were preserved.':''}`);
    }catch{this.say('Could not export the complete preparation data: storage may be unavailable or the backup exceeds 2 MB. Copy important text manually.');}
  }
  private async importBackup(input:HTMLInputElement){
    const file=input.files?.[0];if(!file)return;
    try {
      if(file.size>MAX_PREPARATION_BACKUP_BYTES)throw new Error('File too large');
      const backup=parsePreparationBackup(await file.text());
      const counts=`${Object.keys(backup.coverage.topics).length} checklist entries, ${Object.keys(backup.practice.answers).length} answered questions, ${backup.practice.bookmarks.length} bookmarks, ${Object.keys(backup.revision.recall).length} recall ratings and ${Object.keys(backup.revision.drafts).length} drafts${backup.notebook?`, ${Object.keys(backup.notebook.work).length} PSC work records and ${Object.keys(backup.notebook.mistakes).length} mistake notes`:''}, ${backup.mock?.session?1:0} mock session`;
      if(!confirm(`Replace preparation data with ${counts}? Settings are also replaced. ${backup.mock?'':'This older backup clears the current mock because its matching history is absent. '}${mockNeedsRecovery(backup)?'The mock recovery journal conflicts with this answer history; both will be preserved, and automatic history recording will stop for recovery. ':''}Unknown or invalid records have been discarded.`))return;
      const entries:[string,string][]=[[STUDY_KEY,JSON.stringify(backup.coverage)],[STORAGE_KEY,JSON.stringify(backup.practice)],[REVISION_KEY,JSON.stringify(backup.revision)]];
      if(backup.notebook)entries.push([NOTEBOOK_KEY,JSON.stringify(backup.notebook)]);
      entries.push([MOCK_KEY,JSON.stringify(backup.mock??emptyMock())]);
      await withPreparationLock(()=>writePreparationStores(localStorage,entries));
      this.memoryOnly=false;this.state=backup.revision;this.cardIndex=0;this.renderAll();this.say(`Imported ${counts}. Other open preparation pages refresh through browser storage events.`);
    }catch{this.say('Import failed. Use a complete preparation backup under 2 MB. No imported state was applied in this page; if storage failed, keep your previous export.');}
    finally{input.value='';}
  }
}
if(!customElements.get('revision-studio'))customElements.define('revision-studio',RevisionStudio);
