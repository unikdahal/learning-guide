/** Teaching models: end-of-year net inflows, constant annual rate, no tax or inflation. */
export function presentValue(initial: number, annual: number, years: number, percent: number): number[] {
  if (![initial, annual, years, percent].every(Number.isFinite) || initial < 0 || annual < 0 || !Number.isInteger(years) || years < 1 || years > 30 || percent < 0 || percent > 100) {
    throw new RangeError('Use non-negative cash flows, 1–30 whole years, and a 0–100% rate.');
  }
  return [-initial, ...Array.from({length:years}, (_, i) => annual / (1 + percent / 100) ** (i + 1))];
}

/** Fixed five-activity DAG: A→B→D, A→C→D, A→C→E; both D and E precede finish. */
export function projectSchedule(cDuration: number) {
  if (!Number.isInteger(cDuration) || cDuration < 1 || cDuration > 12) throw new RangeError('C must take 1–12 whole days.');
  const tasks = [
    {id:'A', duration:2, predecessors:[] as string[]},
    {id:'B', duration:4, predecessors:['A']},
    {id:'C', duration:cDuration, predecessors:['A']},
    {id:'D', duration:2, predecessors:['B','C']},
    {id:'E', duration:3, predecessors:['C']},
  ];
  const rows = tasks.map(t => ({...t, es:0, ef:0, ls:0, lf:0, float:0}));
  for (const row of rows) {
    row.es = Math.max(0, ...rows.filter(r => row.predecessors.includes(r.id)).map(r => r.ef));
    row.ef = row.es + row.duration;
  }
  const duration = Math.max(...rows.map(r => r.ef));
  for (const row of [...rows].reverse()) {
    const successors = rows.filter(r => r.predecessors.includes(row.id));
    row.lf = successors.length ? Math.min(...successors.map(r => r.ls)) : duration;
    row.ls = row.lf - row.duration;
    row.float = row.ls - row.es;
  }
  return {duration, rows};
}
