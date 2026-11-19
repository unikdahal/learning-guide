import { withPreparationLock } from '../../lib/preparation-lock.ts';
import { practiceQuestions, practiceTopics } from '../../data/preparation-practice.ts';
import type { PracticeQuestion } from '../../data/preparation-practice.ts';
import { STORAGE_KEY, emptyState, parseState, recordAnswer, selectQuestions, topicPerformance } from './state.ts';
import type { PracticeState, ReviewMode } from './state.ts';

class NecPractice extends HTMLElement {
  private state = emptyState();
  private queue: PracticeQuestion[] = [];
  private position = 0;
  private round = new Map<string, number>();
  private deadline = 0;
  private expired = false;
  private finished = false;
  private interval?: ReturnType<typeof setInterval>;
  private events?: AbortController;
  private storageAvailable = true;
  private $<T extends HTMLElement = HTMLElement>(selector: string): T {
    const element = this.querySelector<T>(selector);
    if (!element) throw new Error(`Missing practice element: ${selector}`);
    return element;
  }
  connectedCallback() {
    if (this.events) return;
    this.events = new AbortController();
    const signal = this.events.signal;
    this.state = this.read();
    this.$('[data-subject]').addEventListener('change', () => this.startRound(), { signal });
    this.$('[data-section]').addEventListener('change', () => this.startRound(), { signal });
    this.$('[data-search]').addEventListener('input', () => this.startRound(), { signal });
    this.$('[data-review]').addEventListener('change', () => this.startRound(), { signal });
    this.$('[data-timing]').addEventListener('change', () => this.startRound(), { signal });
    this.$('[data-restart]').addEventListener('click', () => this.startRound(true), { signal });
    this.$('[data-show-all]').addEventListener('click', () => {
      this.$<HTMLSelectElement>('[data-subject]').value = 'all';
      this.$<HTMLSelectElement>('[data-review]').value = 'all';
      this.$<HTMLSelectElement>('[data-section]').value = 'all';
      this.$<HTMLInputElement>('[data-search]').value = '';
      this.startRound(true);
    }, { signal });
    this.$('[data-form]').addEventListener('submit', event => { event.preventDefault(); this.submit(); }, { signal });
    this.$('[data-bookmark]').addEventListener('click', async () => {
      const q = this.queue[this.position]; if (!q) return;
      await this.update(state => ({ ...state, bookmarks: state.bookmarks.includes(q.id) ? state.bookmarks.filter(id => id !== q.id) : [...state.bookmarks, q.id] }));
      if (this.$<HTMLSelectElement>('[data-review]').value === 'bookmarks') this.startRound();
      else this.renderBookmark();
    }, { signal });
    this.$('[data-previous]').addEventListener('click', () => {
      if (this.position > 0) { this.position--; this.finished = false; this.render(true); }
    }, { signal });
    this.$('[data-next]').addEventListener('click', () => {
      if (this.position < this.queue.length - 1) { this.position++; this.render(true); }
      else {
        this.finished = true; this.stopClock(); this.render();
        this.say(`Round finished. ${this.roundSummary()}. Unanswered questions were not marked wrong. Restart to try again; your history is kept.`);
      }
    }, { signal });
    window.addEventListener('storage', event => {
      if (event.key === STORAGE_KEY || event.key === null) {
        this.state = this.read(); this.renderPerformance(); this.renderBookmark();
      }
    }, { signal });
    this.startRound();
    this.$('[data-interactive]').hidden = false;
    this.$('[data-static]').hidden = true;
  }
  disconnectedCallback() { this.stopClock(); this.events?.abort(); this.events = undefined; }
  private read(): PracticeState {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      return raw && raw.length > 100_000 ? emptyState() : parseState(raw);
    } catch { this.storageAvailable = false; return this.state; }
  }
  private async update(change: (state: PracticeState) => PracticeState) {
    await withPreparationLock(() => {
    const current = this.storageAvailable ? this.read() : this.state;
    this.state = change(current);
    if (this.storageAvailable) {
      try { localStorage.setItem(STORAGE_KEY, JSON.stringify(this.state)); }
      catch { this.storageAvailable = false; }
    }
    this.renderPerformance();
    });
  }
  private say(message: string) { this.$('[data-status]').textContent = message; }
  private stopClock() { if (this.interval) clearInterval(this.interval); this.interval = undefined; this.deadline = 0; }
  private startRound(focus = false) {
    this.stopClock(); this.expired = false; this.finished = false; this.round.clear(); this.position = 0;
    this.queue = selectQuestions(this.state, this.$<HTMLSelectElement>('[data-subject]').value, this.$<HTMLSelectElement>('[data-review]').value as ReviewMode, this.$<HTMLSelectElement>('[data-section]').value, this.$<HTMLInputElement>('[data-search]').value);
    const minutes = Number(this.$<HTMLSelectElement>('[data-timing]').value);
    this.$('[data-clock]').textContent = minutes ? 'Ready' : 'Untimed';
    // Timing begins with the first answer selection, so reading controls uses no practice time.
    this.say(this.queue.length ? `${this.queue.length} questions ready. ${minutes ? 'Your practice timer starts when you select your first answer.' : 'Take your time and use the explanations.'}` : 'No matching questions. Try another question set.');
    this.$('[data-restart]').textContent = 'Restart round';
    this.render(focus); this.renderPerformance();
  }
  private beginClock() {
    const minutes = Number(this.$<HTMLSelectElement>('[data-timing]').value);
    if (!minutes || this.deadline || this.expired || this.finished) return;
    this.deadline = Date.now() + minutes * 60_000;
    this.tick(); this.interval = setInterval(() => this.tick(), 1000);
  }
  private tick() {
    if (!this.deadline) return;
    const seconds = Math.max(0, Math.ceil((this.deadline - Date.now()) / 1000));
    this.$('[data-clock]').textContent = `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')} left`;
    if (seconds === 0) {
      this.stopClock(); this.expired = true; this.render();
      this.say(`Practice time ended. ${this.roundSummary()}. Unanswered questions were not marked wrong. Restart for another round.`);
    }
  }
  private async submit() {
    if (this.deadline && Date.now() >= this.deadline) this.tick();
    const q = this.queue[this.position];
    if (!q || this.expired || this.finished || this.round.has(q.id)) return;
    const checked = this.querySelector<HTMLInputElement>('input[name="practice-answer"]:checked');
    if (!checked) { this.say('Choose an answer before checking.'); this.querySelector<HTMLInputElement>('input[name="practice-answer"]')?.focus(); return; }
    const selected = Number(checked.value);
    this.round.set(q.id, selected);
    await this.update(state => recordAnswer(state, q, selected));
    this.render();
    this.$<HTMLButtonElement>('[data-next]').focus();
  }
  private roundSummary() {
    const correct = this.queue.filter(q => this.round.get(q.id) === q.answer).length;
    return `${correct} correct from ${this.round.size} submitted in this round`;
  }
  private renderBookmark() {
    const q = this.queue[this.position]; if (!q) return;
    const marked = this.state.bookmarks.includes(q.id);
    this.$('[data-bookmark]').setAttribute('aria-pressed', String(marked));
    this.$('[data-bookmark]').textContent = marked ? 'Bookmarked ✓' : 'Bookmark';
  }
  private render(focus = false) {
    const q = this.queue[this.position];
    this.$('[data-card]').hidden = !q; this.$('[data-empty]').hidden = !!q;
    if (!q) return;
    const topic = practiceTopics.find(t => t[0] === q.topic)!;
    this.$('[data-position]').textContent = `Question ${this.position + 1} / ${this.queue.length}`;
    this.$('[data-tag]').textContent = `${topic[1]} · NEC ${q.section} · Original`;
    this.$('[data-prompt]').textContent = q.prompt;
    const selected = this.round.get(q.id);
    const answered = selected !== undefined;
    const options = this.$('[data-options]'); options.replaceChildren();
    q.options.forEach((text, index) => {
      const label = document.createElement('label'); label.className = 'practice-option';
      const input = document.createElement('input'); input.type = 'radio'; input.name = 'practice-answer'; input.value = String(index);
      input.checked = index === selected; input.disabled = answered || this.expired || this.finished;
      input.addEventListener('change', () => this.beginClock());
      const content = document.createElement('span'); content.textContent = `${String.fromCharCode(65 + index)}. ${text}`;
      label.append(input, content);
      if (answered && index === q.answer) { label.dataset.result = 'correct'; content.append(' · Correct answer'); }
      else if (answered && index === selected) { label.dataset.result = 'incorrect'; content.append(' · Your answer'); }
      options.append(label);
    });
    this.$<HTMLButtonElement>('[data-check]').disabled = answered || this.expired || this.finished;
    const feedback = this.$('[data-feedback]'); feedback.hidden = !answered;
    feedback.replaceChildren();
    if (answered) {
      const heading = document.createElement('strong'); heading.textContent = selected === q.answer ? 'Correct. Here’s why.' : `Review this one. Answer ${String.fromCharCode(65 + q.answer)} is correct.`;
      const explanation = document.createElement('p'); explanation.textContent = q.explanation;
      feedback.append(heading, explanation);
    }
    const study = this.$<HTMLAnchorElement>('[data-study]'); study.href = `${this.dataset.base}/nec/chapters/${topic[3]}/`;
    this.$('[data-round-score]').textContent = `${this.round.size} / ${this.queue.length} submitted`;
    this.$<HTMLButtonElement>('[data-previous]').disabled = this.position === 0;
    this.$<HTMLButtonElement>('[data-next]').disabled = this.finished;
    this.$('[data-next]').textContent = this.position === this.queue.length - 1 ? 'Finish round' : 'Next →';
    this.renderBookmark();
    if (focus) this.$('[data-prompt]').focus();
  }
  private renderPerformance() {
    const list = this.$('[data-performance]'); list.replaceChildren();
    const performance = topicPerformance(this.state);
    for (const row of performance) {
      const item = document.createElement('li');
      const label = document.createElement('span'); label.className = 'practice-topic-name'; label.textContent = row.label;
      const value = document.createElement('span'); value.textContent = row.attempts ? `${row.correct}/${row.attempts} correct · ${row.seen}/${row.total} seen` : `No answers yet · 0/${row.total} seen`;
      const bar = document.createElement('div'); bar.className = 'practice-bar'; bar.setAttribute('aria-hidden', 'true');
      if (row.attempts) { const fill = document.createElement('span'); fill.style.width = `${100 * row.correct / row.attempts}%`; bar.append(fill); }
      item.append(label, value, bar); list.append(item);
    }
    this.$('[data-total]').textContent = `${performance.reduce((n, row) => n + row.seen, 0)} / ${practiceQuestions.length} seen`;
    this.$('[data-storage]').textContent = this.storageAvailable ? 'Saved on this browser only. History includes repeat submissions; restarting never erases it.' : 'Browser storage is unavailable. You can keep practising; progress is held only while this page stays open.';
  }
}
if (!customElements.get('nec-practice')) customElements.define('nec-practice', NecPractice);
