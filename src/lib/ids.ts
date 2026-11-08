// Request-local, deterministic prefixes: repeated diagrams never share labels or radio groups.
const counters = new WeakMap<object, Map<string, number>>();
export function uniqueId(locals: object, prefix: string): string {
  let counts=counters.get(locals);
  if (!counts) {counts=new Map();counters.set(locals,counts);}
  const n=(counts.get(prefix) ?? 0)+1;counts.set(prefix,n);
  return `${prefix}-${n}`;
}
