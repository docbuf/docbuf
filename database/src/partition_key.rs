use docbuf_core::uuid::Uuid;
use xxhash_rust::const_xxh3::xxh3_128;

/// Default number of partitions, if none are provided to the partition key's bucket method.
/// This is used to determine the maximum number of partitions to store documents in.
pub const DEFAULT_NUM_PARTITIONS: u16 = 128 * 128;

#[derive(Debug, Clone)]
pub struct PartitionKey(pub(crate) [u8; 16]);

impl From<Uuid> for PartitionKey {
    fn from(value: Uuid) -> Self {
        Self(value.as_bytes().to_owned())
    }
}

impl Into<u128> for &PartitionKey {
    fn into(self) -> u128 {
        u128::from_le_bytes(self.0)
    }
}

impl PartitionKey {
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn as_u128(&self) -> u128 {
        self.into()
    }

    /// Return the partition bucket key based on the partition key.
    /// Used by the database to determine which partition to store the document in.
    pub fn bucket(&self, partitions: Option<u16>) -> u16 {
        let partitions = partitions.unwrap_or(DEFAULT_NUM_PARTITIONS);
        let key = self.as_u128();
        let bucket = key % partitions as u128;
        bucket as u16
    }
}

impl From<String> for PartitionKey {
    fn from(value: String) -> PartitionKey {
        PartitionKey::from(xxh3_128(value.as_bytes()))
    }
}

impl From<&str> for PartitionKey {
    fn from(value: &str) -> PartitionKey {
        PartitionKey::from(xxh3_128(value.as_bytes()))
    }
}

impl From<Vec<u8>> for PartitionKey {
    fn from(value: Vec<u8>) -> PartitionKey {
        PartitionKey::from(value.as_slice())
    }
}

impl From<&[u8]> for PartitionKey {
    fn from(value: &[u8]) -> PartitionKey {
        PartitionKey::from(xxh3_128(value))
    }
}

impl From<u64> for PartitionKey {
    fn from(value: u64) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<i64> for PartitionKey {
    fn from(value: i64) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<u128> for PartitionKey {
    fn from(value: u128) -> PartitionKey {
        PartitionKey(value.to_le_bytes())
    }
}

impl From<i128> for PartitionKey {
    fn from(value: i128) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<[u8; 16]> for PartitionKey {
    fn from(value: [u8; 16]) -> PartitionKey {
        PartitionKey(value.to_owned())
    }
}

impl From<u8> for PartitionKey {
    fn from(value: u8) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<i8> for PartitionKey {
    fn from(value: i8) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<u16> for PartitionKey {
    fn from(value: u16) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<i16> for PartitionKey {
    fn from(value: i16) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<u32> for PartitionKey {
    fn from(value: u32) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<i32> for PartitionKey {
    fn from(value: i32) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<f32> for PartitionKey {
    fn from(value: f32) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<f64> for PartitionKey {
    fn from(value: f64) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<bool> for PartitionKey {
    fn from(value: bool) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}

impl From<char> for PartitionKey {
    fn from(value: char) -> PartitionKey {
        PartitionKey::from(value as u128)
    }
}
