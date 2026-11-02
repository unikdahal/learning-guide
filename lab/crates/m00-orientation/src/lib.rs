//! Module 00 lab. Each article's code is an example in `examples/`; shared pieces live here.

/// One order as a row: 64 bytes in memory. Used by the row-vs-column exercises.
#[derive(Clone, Copy)]
pub struct Order {
    pub id: u64,
    pub customer: u64,
    pub product: u64,
    pub amount: i64,
    pub date: u32,
    pub region: u16,
    pub status: u8,
    pub note: [u8; 25],
}

/// `n` deterministic orders.
pub fn orders(n: usize) -> Vec<Order> {
    (0..n as u64)
        .map(|i| Order {
            id: i,
            customer: i % 10_000,
            product: i % 500,
            amount: (i % 1_000) as i64,
            date: 20_000 + (i % 365) as u32,
            region: (i % 5) as u16,
            status: (i % 3) as u8,
            note: [0; 25],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn order_is_64_bytes() {
        assert_eq!(std::mem::size_of::<super::Order>(), 64);
    }
}
