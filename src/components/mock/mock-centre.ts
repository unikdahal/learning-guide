import { withPreparationLock } from '../../lib/preparation-lock';
import { practiceQuestions } from '../../data/preparation-practice';
import { studyChapters } from '../../data/study-map';
import { MOCK_KEY, emptyMock, parseMock, createMock, answerMock, remainingMock, finaliseMock, persistMockFinalisation, mockReport, retryWrong, type MockState, type MockStorage } from '../../lib/mock-state';

class MockCentre extends HTMLElement {
  private state: MockState = emptyMock();
  private storage: MockStorage | null = null;
  private interval?: ReturnType<typeof setInterval>;
  private busy = false;
  private dirty = false;
  private durableRaw: string | null = null;
  private recoveryAttempted = "";
  private events?: AbortController;
  private find<T extends HTMLElement = HTMLElement>(selector: string) { return this.querySelector<T>(selector)!; }
  private notice(message: string) { this.find('[data-notice]').textContent = message; }
  connectedCallback() {
    if (this.events) return;
    this.events = new AbortController();
    try { this.storage = window.localStorage; this.durableRaw = this.storage.getItem(MOCK_KEY); this.state = parseMock(this.durableRaw); }
    catch { this.storage = null; this.notice('Browser storage is unavailable. This round works in memory; reload will lose it and practice history cannot be saved.'); }
    this.find('[data-live]').hidden = false;
    this.find('[data-fallback]').hidden = true;
    this.find<HTMLFormElement>('[data-setup]').addEventListener('submit', event => {
      event.preventDefault(); const form = new FormData(event.currentTarget as HTMLFormElement);
      void this.update(state => { if (state.session && (state.session.finished === null || (!state.session.recorded && this.storage))) throw new Error('Finish and save the current round first.'); return createMock(Number(form.get('count')) as 20 | 60 | 100, String(form.get('seed')).trim(), Number(form.get('minutes')), Number(form.get('penalty')) as 0 | 0.2, Date.now(), crypto.randomUUID()); });
    }, { signal: this.events.signal });
    this.addEventListener('click', this.handleClick, { signal: this.events.signal });
    this.addEventListener('change', this.change, { signal: this.events.signal });
    window.addEventListener('storage', this.sync, { signal: this.events.signal });
    document.addEventListener('visibilitychange', this.tick, { signal: this.events.signal });
    this.interval = setInterval(this.tick, 1000);
    this.render(); this.tick();
  }
  disconnectedCallback() { clearInterval(this.interval); this.events?.abort(); this.events = undefined; }
  private sync = () => { if (this.storage && !this.busy && !this.dirty) { this.durableRaw = this.storage.getItem(MOCK_KEY); this.state = parseMock(this.durableRaw); this.render(); this.tick(); } };
  private async update(action: (state: MockState) => MockState, focus?: string) {
    if (this.busy) return;
    this.busy = true;
    const run = () => {
      if (this.storage && this.dirty && this.storage.getItem(MOCK_KEY) !== this.durableRaw) throw new Error('Saved preparation data changed while this tab had unsaved work. Export or copy your work before reloading');
      if (this.storage && !this.dirty) {
        this.durableRaw = this.storage.getItem(MOCK_KEY);
        const saved = parseMock(this.durableRaw);
        const unsavedFinish = this.state.session && this.state.session.finished !== null && !this.state.session.recorded;
        if (!unsavedFinish || saved.pending || saved.session?.recorded) this.state = saved;
      }
      // An expired session cannot be changed by a click or a delayed event.
      if (this.state.session && this.state.session.finished === null && !remainingMock(this.state.session)) this.state = finaliseMock(this.state, 'expiry');
      this.state = action(this.state);
      this.dirty = true;
      if (this.storage) {
        // Finish pending history before allowing a new round to replace it.
        if (this.state.session?.finished !== null && this.state.session) this.state = persistMockFinalisation(this.storage, this.state);
        else {
          this.storage.setItem(MOCK_KEY, JSON.stringify(this.state));
        }
        this.dirty = false;
        this.notice('Round saved in this browser.');
        this.durableRaw = this.storage.getItem(MOCK_KEY);
      }
    };
    try {
      await withPreparationLock(run);
    } catch (error) {
      this.notice(`Saving failed: ${error instanceof Error ? error.message : 'browser storage error'}. Keep this tab open; use Retry saving.`);
      // Recover the durable journal if a write failed after it was saved.
      try { const saved = this.storage && parseMock(this.storage.getItem(MOCK_KEY)); if (saved?.pending) { this.state = saved; this.durableRaw = this.storage!.getItem(MOCK_KEY); } } catch { /* Keep in-memory work. */ }
    } finally { this.busy = false; this.render(); if (focus) this.find(focus)?.focus(); }
  }
  private tick = () => {
    const s = this.state.session;
    if (!s) return;
    if (s.finished === null) {
      const seconds = Math.ceil(remainingMock(s) / 1000);
      this.find('[data-clock]').textContent = `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')} remaining`;
      if (!seconds) void this.update(state => finaliseMock(state, 'expiry'), '[data-results] h2');
    } else if (!s.recorded && this.storage && !this.busy && this.recoveryAttempted !== s.id) {
      // One recovery attempt on connection, not an endless retry loop on quota errors.
      this.recoveryAttempted = s.id;
      void this.update(state => state);
    }
  };
  private handleClick = (event: Event) => {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>('button');
    if (!button || button.type === 'submit' || this.busy) return;
    const s = this.state.session;
    if (!s) return;
    if (button.hasAttribute('data-submit')) {
      this.find('[data-confirm-count]').textContent = `${Object.keys(s.answers).length} answered; ${s.ids.length - Object.keys(s.answers).length} blank.`;
      this.find('[data-confirm]').hidden = false; this.find('[data-confirm] h3').focus(); return;
    }
    if (button.hasAttribute('data-cancel')) { this.find('[data-confirm]').hidden = true; this.find('[data-submit]').focus(); return; }
    if (button.hasAttribute('data-finish')) { this.find('[data-confirm]').hidden = true; void this.update(state => finaliseMock(state, 'manual'), '[data-results] h2'); return; }
    if ((button.hasAttribute('data-save-again') || button.hasAttribute('data-save-active'))) { void this.update(state => state); return; }
    if (button.hasAttribute('data-new') || button.hasAttribute('data-retry')) {
      if (!s.recorded && this.storage) { this.notice('Save this report’s history before starting another round.'); return; }
      if (button.hasAttribute('data-new')) { this.find('[data-setup]').hidden = false; this.find('[data-results]').hidden = true; this.find<HTMLInputElement>('[name=count]').focus(); }
      else void this.update(state => retryWrong(state.session!, crypto.randomUUID(), (s.deadline - s.started) / 60_000, Date.now(), crypto.randomUUID()), '[data-question] legend');
      return;
    }
    void this.update(state => {
      const current = state.session; if (!current || current.finished !== null) return state;
      if (button.hasAttribute('data-clear')) return answerMock(state, current.ids[current.cursor]!, null);
      if (button.hasAttribute('data-flag')) { const id = current.ids[current.cursor]!; return { ...state, session: { ...current, flagged: current.flagged.includes(id) ? current.flagged.filter(v => v !== id) : [...current.flagged, id] } }; }
      let cursor = current.cursor;
      if (button.hasAttribute('data-index')) cursor = Number(button.dataset.index);
      else if (button.hasAttribute('data-prev')) cursor--;
      else if (button.hasAttribute('data-next')) cursor++;
      return { ...state, session: { ...current, cursor: Math.max(0, Math.min(current.ids.length - 1, cursor)) } };
    }, '[data-question] legend');
  };
  private change = (event: Event) => {
    const input = event.target as HTMLInputElement, s = this.state.session;
    if (input.name === 'mock-answer' && s) { const id = s.ids[s.cursor]!; void this.update(state => answerMock(state, id, Number(input.value)), `[data-question] input[value="${input.value}"]`); }
  };
  private render() {
    const s = this.state.session, active = !!s && s.finished === null;
    this.find('[data-setup]').hidden = !!s;
    this.find('[data-round]').hidden = !active;
    this.find('[data-results]').hidden = !s || active;
    if (!active) this.find('[data-confirm]').hidden = true;
    this.find('[data-save-active]').hidden = !active || !this.dirty || !this.storage;
    if (!s) return;
    if (active) {
      this.find('[data-meta]').textContent = `${s.kind === 'retry' ? 'Wrong-answer retry' : 'Training'} · ${s.ids.length} questions · Seed: ${s.seed} · Correct +1, wrong ${s.penalty ? '−0.2' : '0'}, blank 0`;
      const answered = Object.keys(s.answers).length;
      this.find('[data-progress]').textContent = `${answered} answered · ${s.ids.length - answered} unanswered · ${s.flagged.length} flagged`;
      const nav = this.find('[data-navigator]'); nav.replaceChildren();
      s.ids.forEach((id, index) => {
        const b = document.createElement('button'); b.type = 'button'; b.dataset.index = String(index);
        const status = s.answers[id] === undefined ? 'unanswered' : 'answered';
        b.textContent = `${index + 1} ${status}${s.flagged.includes(id) ? ' · flagged' : ''}`;
        if (index === s.cursor) b.setAttribute('aria-current', 'true'); nav.append(b);
      });
      const id = s.ids[s.cursor]!, q = practiceQuestions.find(q => q.id === id)!;
      const container = this.find('[data-question]'); container.replaceChildren();
      const field = document.createElement('fieldset'), legend = document.createElement('legend'); legend.tabIndex = -1;
      legend.textContent = `Question ${s.cursor + 1} · NEC ${q.section}: ${q.prompt}`; field.append(legend);
      q.options.forEach((text, index) => { const label = document.createElement('label'), radio = document.createElement('input'); radio.type = 'radio'; radio.name = 'mock-answer'; radio.value = String(index); radio.checked = s.answers[id] === index; label.append(radio, document.createTextNode(text)); field.append(label); });
      container.append(field);
      for (const [key, text] of [['clear', 'Clear answer'], ['flag', s.flagged.includes(id) ? 'Unflag question' : 'Flag question']]) { const b = document.createElement('button'); b.type = 'button'; b.setAttribute(`data-${key}`, ''); b.textContent = text!; if (key === 'flag') b.setAttribute('aria-pressed', String(s.flagged.includes(id))); container.append(b); }
      this.find<HTMLButtonElement>('[data-prev]').disabled = s.cursor === 0;
      this.find<HTMLButtonElement>('[data-next]').disabled = s.cursor === s.ids.length - 1;
      const sec = Math.ceil(remainingMock(s) / 1000); this.find('[data-clock]').textContent = `${Math.floor(sec / 60)}:${String(sec % 60).padStart(2, '0')} remaining`;
    } else {
      const report = mockReport(s);
      this.find('[data-summary]').textContent = `${s.reason === 'expiry' ? 'Time expired' : 'Submitted'} · ${report.correct} correct · ${report.wrong} wrong · ${report.blank} blank · Score ${report.score} / ${s.ids.length} (correct +1, wrong ${s.penalty ? '−0.2' : '0'}, blank 0). Seed: ${s.seed}`;
      this.find('[data-saved]').textContent = s.recorded ? 'Answered questions recorded once in practice history. Blank questions do not add attempts.' : this.storage ? 'Practice history is not yet saved.' : 'In-memory report only: browser storage unavailable; practice history was not saved.';
      this.find<HTMLButtonElement>('[data-retry]').disabled = report.wrong === 0;
      this.find('[data-save-again]').hidden = s.recorded || !this.storage;
      const body = this.find('[data-breakdown]'); body.replaceChildren();
      for (const c of report.chapters) { const row = document.createElement('tr'); for (const value of [`${c.chapter}. ${studyChapters.find(v => v.id === c.chapter)?.title}`, c.correct, c.wrong, c.blank, c.score]) { const cell = document.createElement('td'); cell.textContent = String(value); row.append(cell); } body.append(row); }
      const review = this.find('[data-review]'); review.replaceChildren();
      report.rows.forEach((row, index) => {
        const d = document.createElement('details'), summary = document.createElement('summary'); summary.textContent = `${index + 1}. NEC ${row.question.section} · ${row.outcome}: ${row.question.prompt}`; d.append(summary);
        for (const text of [`Your answer: ${row.selected === undefined ? 'Blank' : row.question.options[row.selected]}`, `Correct answer: ${row.question.options[row.question.answer]}`, row.question.explanation]) { const p = document.createElement('p'); p.textContent = text; d.append(p); } review.append(d);
      });
    }
  }
}
if (!customElements.get('mock-centre')) customElements.define('mock-centre', MockCentre);
