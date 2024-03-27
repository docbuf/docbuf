use super::*;

use serde_derive::{Deserialize, Serialize};

pub const VTABLE_FIELD_OFFSET_SIZE_BYTES: usize = 18;

// Field Offset range of the resulting document buffer bytes
pub type VTableFieldOffsetIndex = (VTableItemIndex, VTableFieldIndex);

#[derive(Debug, Default, Clone)]
pub struct VTableFieldOffset(pub VTableFieldOffsetIndex, pub Range<usize>);

impl VTableFieldOffset {
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.1.clone()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.1.len()
    }

    #[inline]
    pub fn as_bytes(&self) -> [u8; VTABLE_FIELD_OFFSET_SIZE_BYTES] {
        let mut bytes = [0; VTABLE_FIELD_OFFSET_SIZE_BYTES];
        bytes[0..1].copy_from_slice(&self.0 .0.to_le_bytes());
        bytes[1..2].copy_from_slice(&self.0 .1.to_le_bytes());
        bytes[2..10].copy_from_slice(&self.1.start.to_le_bytes());
        bytes[10..18].copy_from_slice(&self.1.end.to_le_bytes());
        bytes
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let indexes = &bytes[0..2];
        let range_start = &bytes[2..10];
        let range_end = &bytes[10..8];

        Self(
            (
                VTableItemIndex::from_le_bytes([indexes[0]]),
                VTableFieldIndex::from_le_bytes([indexes[1]]),
            ),
            Range {
                start: usize::from_le_bytes([
                    range_start[0],
                    range_start[1],
                    range_start[2],
                    range_start[3],
                    range_start[4],
                    range_start[5],
                    range_start[6],
                    range_start[7],
                ]),
                end: usize::from_le_bytes([
                    range_end[0],
                    range_end[1],
                    range_end[2],
                    range_end[3],
                    range_end[4],
                    range_end[5],
                    range_end[6],
                    range_end[7],
                ]),
            },
        )
    }
}

#[derive(Debug, Clone)]
pub enum VTableFieldOffsetDiff {
    Increase(usize),
    Decrease(usize),
    None,
}

impl VTableFieldOffsetDiff {
    #[inline]
    pub fn new(old_len: usize, new_len: usize) -> Self {
        if new_len == old_len {
            Self::None
        } else if new_len > old_len {
            Self::Increase(new_len - old_len)
        } else {
            Self::Decrease(old_len - new_len)
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct VTableFieldOffsets(Vec<VTableFieldOffset>);

impl VTableFieldOffsets {
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0
            .iter()
            .flat_map(|offset| offset.as_bytes())
            .collect::<Vec<u8>>()
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut offsets = Vec::new();
        let mut index = 0;

        while index < bytes.len() {
            let offset = VTableFieldOffset::from_bytes(
                &bytes[index..index + VTABLE_FIELD_OFFSET_SIZE_BYTES],
            );
            offsets.push(offset);
            index += VTABLE_FIELD_OFFSET_SIZE_BYTES;
        }

        Self(offsets)
    }

    #[inline]
    pub fn with_capacity(size: usize) -> Self {
        Self(Vec::with_capacity(size))
    }

    #[inline]
    pub fn push(&mut self, offset: VTableFieldOffset) {
        match self
            .0
            .iter_mut()
            .find(|existing_offset| existing_offset.0 == offset.0)
            .as_mut()
        {
            Some(existing_offset) => {
                existing_offset.1.end = offset.1.end;
            }
            None => {
                self.0.push(offset);
            }
        }
    }

    #[inline]
    pub fn resize(&mut self, from_index: usize, diff: VTableFieldOffsetDiff) {
        self.0
            .iter_mut()
            .filter(|offset| offset.1.start >= from_index)
            .for_each(|offset| match diff {
                VTableFieldOffsetDiff::Increase(increase) => {
                    if offset.1.start != from_index {
                        offset.1.start += increase;
                    }

                    offset.1.end += increase;
                }
                VTableFieldOffsetDiff::Decrease(decrease) => {
                    if offset.1.start != from_index {
                        offset.1.start -= decrease;
                    }

                    offset.1.end -= decrease;
                }
                VTableFieldOffsetDiff::None => {}
            });
    }
}

impl AsMut<Vec<VTableFieldOffset>> for VTableFieldOffsets {
    #[inline]
    fn as_mut(&mut self) -> &mut Vec<VTableFieldOffset> {
        &mut self.0
    }
}

impl AsRef<Vec<VTableFieldOffset>> for VTableFieldOffsets {
    #[inline]
    fn as_ref(&self) -> &Vec<VTableFieldOffset> {
        &self.0
    }
}

impl VTableField {
    #[inline]
    pub fn as_offset(&self, range: Range<usize>) -> VTableFieldOffset {
        VTableFieldOffset((self.item_index, self.index), range)
    }
}
