///! Optimistic lock client implementation

use crate::{Result, Error, RollupTx, Cond};

use alloc::{
    vec::Vec,
    string::{String, ToString},
    collections::BTreeMap
};
use scale::Encode;
use primitive_types::U256;

pub const GLOBAL: &str = "Global";

pub type LockId = u8;
pub type LockVersion = u32;

pub struct Locks {
    num_locks: LockId,
    lock_ids: BTreeMap<String, LockId>,
    deps: BTreeMap<LockId, LockId>,
}

impl Default for Locks {
    fn default() -> Self {
        let mut lock_ids = BTreeMap::new();
        lock_ids.insert(GLOBAL.to_string(), 0);
        Self {
            num_locks: 1,
            lock_ids,
            deps: BTreeMap::new(),
        }
    }
}

impl Locks {
    pub fn add(&mut self, lock: &str, parent: &str) -> Result<LockId> {
        let parent_id = *self.lock_ids.get(parent).ok_or(Error::UnknownLock)?;
        let id = self.num_locks;
        self.lock_ids.insert(lock.to_string(), id);
        self.deps.insert(id, parent_id);
        self.num_locks += 1;
        Ok(id)
    }

    // TODO: support parameterized lock
    // TODO: dedup tx entries

    pub fn tx_read(&self, tx: &mut RollupTx, reader: &impl LockVersionReader, lock: &str) -> Result<()> {
        let id = *self.lock_ids.get(lock).ok_or(Error::UnknownLock)?;
        // Only check version
        let v = reader.get_version(id)?;
        tx.conds.push(Cond::Eq(key(id), Some(value(v))));
        Ok(())
    }

    pub fn tx_write(&self, tx: &mut RollupTx, reader: &impl LockVersionReader, lock: &str) -> Result<()> {
        let id = *self.lock_ids.get(lock).ok_or(Error::UnknownLock)?;
        // Check reading version
        let v = reader.get_version(id)?;
        tx.conds.push(Cond::Eq(key(id), Some(value(v))));
        // Update writing versions
        let mut i = Some(id);
        while let Some(id) = i {
            let v = reader.get_version(id)?;
            tx.updates.push((key(id), Some(value(v + 1))));
            i = self.deps.get(&id).cloned();
        }
        Ok(())
    }
}

pub trait LockVersionReader {
    fn get_version(&self, id: LockId) -> Result<LockVersion>;
}

fn key(id: LockId) -> Vec<u8> {
    Encode::encode(&id)
}
fn value(version: LockVersion) -> Vec<u8> {
    let x = U256::from(version);
    Encode::encode(&x)
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Default)]
    struct MockVersionStore {
        versions: BTreeMap<LockId, LockVersion>,
    }
    impl LockVersionReader for MockVersionStore {
        fn get_version(&self, id: LockId) -> Result<LockVersion> {
            Ok(self.versions.get(&id).cloned().unwrap_or(0))
        }
    }

    #[test]
    fn lock_works() {
        let mut locks = Locks::default();
        locks.add("a", GLOBAL).unwrap();
        locks.add("b", "a").unwrap();

        let mut vstore = MockVersionStore::default();

        // Simple read
        let mut tx = RollupTx::default();
        locks.tx_read(&mut tx, &vstore, "a")
            .expect("read should succeed");
        assert_eq!(tx, RollupTx {
            conds: vec![
                Cond::Eq(key(1), Some(value(0))),
            ],
            actions: vec![],
            updates: vec![],
        });

        // Read & write
        let mut tx = RollupTx::default();
        locks.tx_write(&mut tx, &vstore, "a")
            .expect("read should succeed");
        assert_eq!(tx, RollupTx {
            conds: vec![
                Cond::Eq(key(1), Some(value(0))),
            ],
            actions: vec![],
            updates: vec![
                (key(1), Some(value(1))),
                (key(0), Some(value(1))),
            ],
        });
    }
}