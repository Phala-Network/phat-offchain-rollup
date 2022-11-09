use alloc::collections::BTreeSet;

use crate::traits::AccessTracking;

pub type ReadTracker<K> = AccessTracker<K, false>;
pub type RwTracker<K> = AccessTracker<K, true>;

/// Tracker that always emits the given one access history
pub struct OneLock<Key>(pub Key);

impl<Key> AccessTracking for OneLock<Key> {
    type Key = Key;

    fn read(&mut self, _key: &Self::Key) {}

    fn write(&mut self, _key: &Self::Key) {}

    fn collect_into(self) -> alloc::vec::Vec<Self::Key> {
        alloc::vec![self.0]
    }
}

/// Tracker that emit read and(optional) write access history
pub struct AccessTracker<Key, const TRACK_WRITE: bool> {
    track: BTreeSet<Key>,
}

impl<Key, const TRACK_WRITE: bool> AccessTracker<Key, TRACK_WRITE> {
    pub fn new() -> Self {
        Self {
            track: Default::default(),
        }
    }
}

impl<Key, const TRACK_WRITE: bool> AccessTracking for AccessTracker<Key, TRACK_WRITE>
where
    Key: Ord + Clone,
{
    type Key = Key;

    fn read(&mut self, key: &Self::Key) {
        self.track.insert(key.clone());
    }

    fn write(&mut self, key: &Self::Key) {
        if TRACK_WRITE {
            self.track.insert(key.clone());
        }
    }

    fn collect_into(self) -> alloc::vec::Vec<Self::Key> {
        self.track.into_iter().collect()
    }
}
