// A small, dependency-free line chart for measured data in articles.
// Renders SVG at the container's real width (so text stays 12px on phones), with log or linear
// axes, a legend for two or more series, a crosshair that snaps to the nearest measured x and
// lists every series there, optional vertical markers, and a table view for screen readers and
// for anyone who wants the numbers.

export type Series = { name: string; color: string; points: [number, number][] };
export type Tick = { v: number; label: string };
export type Marker = { x: number; label: string };

export type ChartOptions = {
  series: Series[];
  xLog?: boolean;
  yLog?: boolean;
  xTicks: Tick[];
  yTicks: Tick[];
  xLabel: string;
  yLabel: string;
  fmtX: (x: number) => string;
  fmtY: (y: number) => string;
  markers?: Marker[];
  height?: number;
  /** Highlighted x (for example, chosen by a slider elsewhere). */
  focusX?: number;
};

const NS = 'http://www.w3.org/2000/svg';

function el<K extends keyof SVGElementTagNameMap>(tag: K, attrs: Record<string, string | number>): SVGElementTagNameMap[K] {
  const e = document.createElementNS(NS, tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, String(v));
  return e;
}

const observers = new WeakMap<HTMLElement, ResizeObserver>();

export function renderLineChart(host: HTMLElement, opts: ChartOptions): void {
  host.classList.add('qe-chart');
  observers.get(host)?.disconnect(); // re-rendering: drop the previous chart's observer
  host.replaceChildren();

  if (opts.series.length > 1) {
    const legend = document.createElement('ul');
    legend.className = 'qe-chart-legend';
    for (const s of opts.series) {
      const li = document.createElement('li');
      const key = document.createElement('span');
      key.className = 'key';
      key.style.background = s.color;
      li.append(key, document.createTextNode(s.name));
      legend.append(li);
    }
    host.append(legend);
  }

  const plot = document.createElement('div');
  plot.className = 'qe-chart-plot';
  host.append(plot);
  const tip = document.createElement('div');
  tip.className = 'qe-chart-tip';
  tip.hidden = true;
  plot.append(tip);

  const draw = () => {
    plot.querySelector('svg')?.remove();
    const W = Math.max(280, plot.clientWidth);
    const H = opts.height ?? 260;
    const m = { l: 52, r: 14, t: 12, b: 44 };
    const iw = W - m.l - m.r;
    const ih = H - m.t - m.b;
    const allX = opts.series.flatMap((s) => s.points.map((p) => p[0]));
    const xs = opts.xTicks.map((t) => t.v).concat(allX);
    const ys = opts.yTicks.map((t) => t.v);
    const fx = opts.xLog ? Math.log10 : (v: number) => v;
    const fy = opts.yLog ? Math.log10 : (v: number) => v;
    const [x0, x1] = [Math.min(...xs.map(fx)), Math.max(...xs.map(fx))];
    const [y0, y1] = [Math.min(...ys.map(fy)), Math.max(...ys.map(fy))];
    const sx = (v: number) => m.l + ((fx(v) - x0) / (x1 - x0)) * iw;
    const sy = (v: number) => m.t + ih - ((fy(v) - y0) / (y1 - y0)) * ih;

    const svg = el('svg', { viewBox: `0 0 ${W} ${H}`, width: W, height: H, role: 'img', 'aria-label': `${opts.yLabel} by ${opts.xLabel}` });
    for (const t of opts.yTicks) {
      svg.append(el('line', { class: 'grid', x1: m.l, x2: W - m.r, y1: sy(t.v), y2: sy(t.v) }));
      const tx = el('text', { class: 'tick', x: m.l - 6, y: sy(t.v) + 4, 'text-anchor': 'end' });
      tx.textContent = t.label;
      svg.append(tx);
    }
    for (const t of opts.xTicks) {
      // Keep edge labels inside the plot: anchor them inward when they sit near either side.
      const x = sx(t.v);
      const anchor = x > W - m.r - 24 ? 'end' : x < m.l + 8 ? 'start' : 'middle';
      const tx = el('text', { class: 'tick', x: anchor === 'end' ? Math.min(x, W - 2) : x, y: H - m.b + 16, 'text-anchor': anchor });
      tx.textContent = t.label;
      svg.append(tx);
    }
    svg.append(el('line', { class: 'axis', x1: m.l, x2: W - m.r, y1: m.t + ih, y2: m.t + ih }));
    const xl = el('text', { class: 'label', x: m.l + iw / 2, y: H - 6, 'text-anchor': 'middle' });
    xl.textContent = opts.xLabel;
    const yl = el('text', { class: 'label', x: 12, y: m.t + ih / 2, transform: `rotate(-90 12 ${m.t + ih / 2})`, 'text-anchor': 'middle' });
    yl.textContent = opts.yLabel;
    svg.append(xl, yl);

    for (const mk of opts.markers ?? []) {
      const x = sx(mk.x);
      svg.append(el('line', { class: 'marker', x1: x, x2: x, y1: m.t, y2: m.t + ih }));
      const t = el('text', { class: 'marker-label', x: x + 4, y: m.t + 11 });
      t.textContent = mk.label;
      svg.append(t);
    }

    for (const s of opts.series) {
      const d = s.points.map((p, i) => `${i ? 'L' : 'M'}${sx(p[0]).toFixed(1)},${sy(p[1]).toFixed(1)}`).join('');
      svg.append(el('path', { class: 'line', d, style: `stroke: ${s.color}` }));
      const last = s.points[s.points.length - 1];
      svg.append(el('circle', { class: 'dot', cx: sx(last[0]), cy: sy(last[1]), r: 4, style: `fill: ${s.color}` }));
    }

    if (opts.focusX !== undefined) {
      const x = sx(opts.focusX);
      svg.append(el('line', { class: 'focus', x1: x, x2: x, y1: m.t, y2: m.t + ih }));
      for (const s of opts.series) {
        const p = s.points.find((q) => q[0] === opts.focusX);
        if (p) svg.append(el('circle', { class: 'dot', cx: x, cy: sy(p[1]), r: 5, style: `fill: ${s.color}` }));
      }
    }

    const hair = el('line', { class: 'hair', x1: 0, x2: 0, y1: m.t, y2: m.t + ih, visibility: 'hidden' });
    svg.append(hair);
    const hot = el('rect', { x: m.l, y: m.t, width: iw, height: ih, fill: 'transparent' });
    svg.append(hot);
    const uniqueX = [...new Set(allX)].sort((a, b) => a - b);
    const show = (clientX: number) => {
      const r = svg.getBoundingClientRect();
      const px = ((clientX - r.left) / r.width) * W;
      let best = uniqueX[0];
      for (const x of uniqueX) if (Math.abs(sx(x) - px) < Math.abs(sx(best) - px)) best = x;
      hair.setAttribute('x1', String(sx(best)));
      hair.setAttribute('x2', String(sx(best)));
      hair.setAttribute('visibility', 'visible');
      tip.replaceChildren();
      const head = document.createElement('b');
      head.textContent = opts.fmtX(best);
      tip.append(head);
      for (const s of opts.series) {
        const p = s.points.find((q) => q[0] === best);
        if (!p) continue;
        const row = document.createElement('span');
        const key = document.createElement('i');
        key.style.background = s.color;
        const val = document.createElement('strong');
        val.textContent = opts.fmtY(p[1]);
        row.append(key, val, document.createTextNode(opts.series.length > 1 ? ` ${s.name}` : ''));
        tip.append(row);
      }
      tip.hidden = false;
      const left = Math.min(Math.max(sx(best) / W * plot.clientWidth - tip.offsetWidth / 2, 0), plot.clientWidth - tip.offsetWidth);
      tip.style.left = `${left}px`;
    };
    hot.addEventListener('pointermove', (e) => show(e.clientX));
    hot.addEventListener('pointerdown', (e) => show(e.clientX));
    hot.addEventListener('pointerleave', () => {
      hair.setAttribute('visibility', 'hidden');
      tip.hidden = true;
    });
    plot.prepend(svg);
  };

  draw();
  let w = plot.clientWidth;
  const ro = new ResizeObserver(() => {
    if (Math.abs(plot.clientWidth - w) > 4) {
      w = plot.clientWidth;
      draw();
    }
  });
  ro.observe(plot);
  observers.set(host, ro);

  const details = document.createElement('details');
  details.className = 'qe-chart-table';
  const summary = document.createElement('summary');
  summary.textContent = 'Show the numbers';
  const table = document.createElement('table');
  const head = document.createElement('tr');
  for (const h of [opts.xLabel, ...opts.series.map((s) => s.name)]) {
    const th = document.createElement('th');
    th.textContent = h;
    head.append(th);
  }
  table.append(head);
  const xsAll = [...new Set(opts.series.flatMap((s) => s.points.map((p) => p[0])))].sort((a, b) => a - b);
  for (const x of xsAll) {
    const tr = document.createElement('tr');
    const td = document.createElement('td');
    td.textContent = opts.fmtX(x);
    tr.append(td);
    for (const s of opts.series) {
      const c = document.createElement('td');
      const p = s.points.find((q) => q[0] === x);
      c.textContent = p ? opts.fmtY(p[1]) : '–';
      tr.append(c);
    }
    table.append(tr);
  }
  details.append(summary, table);
  host.append(details);
}
