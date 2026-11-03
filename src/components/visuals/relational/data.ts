// Shared sample data for Module 01's visuals. Identical to lab/crates/m01-relational/src/lib.rs
// (`sample::customer` and `sample::orders`), so every count shown in a visual matches the lab.

export type Customer = { id: number; name: string; segment: string; region: string; updated_at: string };
export type Order = { order_id: number; cust_id: number; amount: number; updated_at: string };

export const customers: Customer[] = [
  { id: 1, name: 'Asha', segment: 'retail', region: 'ASIA', updated_at: '2026-01-05' },
  { id: 2, name: 'Bruno', segment: 'wholesale', region: 'EUROPE', updated_at: '2026-01-19' },
  { id: 3, name: 'Chen', segment: 'retail', region: 'ASIA', updated_at: '2026-02-11' },
  { id: 4, name: 'Dara', segment: 'retail', region: 'AFRICA', updated_at: '2026-03-02' },
  { id: 5, name: 'Emeka', segment: 'government', region: 'AFRICA', updated_at: '2026-03-14' },
  { id: 6, name: 'Farah', segment: 'retail', region: 'EUROPE', updated_at: '2026-03-28' },
];

export const orders: Order[] = [
  { order_id: 101, cust_id: 1, amount: 250, updated_at: '2026-04-01' },
  { order_id: 102, cust_id: 3, amount: 40, updated_at: '2026-04-02' },
  { order_id: 103, cust_id: 1, amount: 90, updated_at: '2026-04-02' },
  { order_id: 104, cust_id: 4, amount: 610, updated_at: '2026-04-03' },
  { order_id: 105, cust_id: 6, amount: 75, updated_at: '2026-04-05' },
  { order_id: 106, cust_id: 9, amount: 120, updated_at: '2026-04-06' },
];

export const customerAttrs = ['id', 'name', 'segment', 'region'] as const;
