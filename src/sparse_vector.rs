use std::slice::SliceIndex;

pub struct SparseVector(Vec<u16>);

impl SparseVector {
    pub fn new(vector: Vec<u16>) -> Self {
        Self(vector)
    }

    pub fn swap(&mut self, first: u16, second: u16) {
        let (first, second) = if first == second {
            return;
        } else if first < second {
            (first, second)
        } else {
            (second, first)
        };

        let maybe_first = self.0.binary_search(&first);
        let maybe_second = self.0.binary_search(&second);

        match (maybe_first, maybe_second) {
            (Ok(_), Ok(_)) => {
                // both are set, so don't need to do anything
            }
            (Ok(first_idx), Err(second_idx)) => {
                self.0.copy_within(first_idx + 1..second_idx, first_idx);
                self.0[second_idx] = first;
            }
            (Err(first_idx), Ok(second_idx)) => {
                self.0.copy_within(first_idx..second_idx - 1, second_idx);
                self.0[first_idx] = second;
            }
            (Err(_), Err(_)) => {
                // neither are set, so don't need to do anything
            }
        }
    }
}

impl std::ops::Deref for SparseVector {
    type Target = Vec<u16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SparseVector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
