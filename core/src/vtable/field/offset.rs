use super::*;

// Field Offset range of the resulting document buffer bytes
pub type VTableFieldOffsetIndex = (VTableItemIndex, VTableFieldIndex);

#[derive(Debug, Default, Clone)]
pub struct VTableFieldOffset(pub VTableFieldOffsetIndex, Range<usize>);

impl VTableFieldOffset {
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.1.clone()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.1.len()
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
    pub fn with_capacity(size: usize) -> Self {
        Self(Vec::with_capacity(size))
    }

    #[inline]
    pub fn push(&mut self, offset: VTableFieldOffset) {
        self.0.push(offset);
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

impl<'a> VTableField<'a> {
    #[inline]
    pub fn as_offset(&self, range: Range<usize>) -> VTableFieldOffset {
        VTableFieldOffset((self.item_index, self.index), range)
    }
}
