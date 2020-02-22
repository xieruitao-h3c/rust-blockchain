use std::cmp::Ordering;
use crate::types::Tx;

pub struct Mempool {
    txs: Vec<Tx>,
}

impl Mempool {
    pub fn new() -> Self {
        Mempool {
            txs: vec![],
        }
    }

    pub fn add(&mut self, tx: Tx) -> bool {
        if self.contains(&tx) {
            return false;
        }

        // if fee is higher, replace
        for (i, t) in self.txs.iter().enumerate() {
            if t.from == tx.from && t.to == tx.to && t.fee < tx.fee {
                self.txs[i] = tx;
                return true;
            }
        }

        // otherwise, add
        self.txs.push(tx);
        true
    }

    pub fn remove(&mut self, tx: &Tx) -> bool {
        if let Some(i) = self.txs.iter().position(|t| *t == *tx) {
            self.txs.remove(i);
            return true;
        }
        false
    }

    pub fn get_all(&mut self) -> &Vec<Tx> {
        self.txs.sort_by(|a, b| {
            if a.fee < b.fee { Ordering::Greater } else { Ordering::Less }
        });
        &self.txs
    }

    pub fn len(&self) -> usize {
        self.txs.len()
    }

    fn contains(&self, tx: &Tx) -> bool {
        self.txs.contains(tx)
    }
}

#[cfg(test)]
mod tests {
    use crate::mempool::Mempool;
    use crate::types::Tx;

    #[test]
    fn test_add_once() {
        let mut mp = Mempool::new();

        let tx1 = Tx {
            from: 'A',
            to: 'B',
            amount: 1,
            fee: 0.123,
        };
        let tx2 = tx1.clone();

        assert_eq!(mp.add(tx1), true);
        assert_eq!(mp.add(tx2), false);
        assert_eq!(mp.len(), 1);
    }

    #[test]
    fn test_add_with_higher_fee() {
        let mut mp = Mempool::new();

        let tx1 = Tx {
            from: 'A',
            to: 'B',
            amount: 1,
            fee: 0.123,
        };
        let mut tx2 = tx1.clone();
        tx2.fee = 0.456;

        assert_eq!(mp.add(tx1), true);
        assert_eq!(mp.add(tx2), true);
        assert_eq!(mp.len(), 1);
        assert_eq!(mp.get_all()[0].fee, 0.456);
    }

    #[test]
    fn test_get_all() {
        let mut mp = Mempool::new();

        mp.add(Tx { from: 'A', to: 'B', amount: 1, fee: 0.234 });
        mp.add(Tx { from: 'B', to: 'C', amount: 1, fee: 0.345 });
        mp.add(Tx { from: 'C', to: 'D', amount: 1, fee: 0.123 });

        let txs = mp.get_all();
        assert_eq!(txs[0].fee, 0.345);
        assert_eq!(txs[1].fee, 0.234);
        assert_eq!(txs[2].fee, 0.123);
    }

    #[test]
    fn test_remove() {
        let mut mp = Mempool::new();
        let tx = Tx { from: 'A', to: 'B', amount: 1, fee: 0.234 };

        assert_eq!(mp.add(tx.clone()), true);
        assert_eq!(mp.remove(&tx), true);
        assert_eq!(mp.remove(&tx), false);
    }
}
