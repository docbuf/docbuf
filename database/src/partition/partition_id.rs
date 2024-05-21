use std::ops::Deref;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PartitionId(u16);

impl AsRef<u16> for PartitionId {
    fn as_ref(&self) -> &u16 {
        &self.0
    }
}

impl Deref for PartitionId {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PartitionKey> for PartitionId {
    fn from(key: PartitionKey) -> Self {
        Self(key.bucket(None))
    }
}

impl From<[u8; 16]> for PartitionId {
    fn from(id: [u8; 16]) -> Self {
        Self::from(PartitionKey::from(id))
    }
}

impl From<u16> for PartitionId {
    fn from(id: u16) -> Self {
        Self(id)
    }
}

impl From<PartitionId> for u16 {
    fn from(id: PartitionId) -> Self {
        id.0
    }
}

impl From<&PartitionId> for u16 {
    fn from(id: &PartitionId) -> Self {
        id.0
    }
}
