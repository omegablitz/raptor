use std::ops::Deref;

use crate::{common, sparse_vector::SparseVector};

/// Sparce Matrix
///
/// Original implementation
/// https://github.com/google/gofountain/blob/master/block.go
///
/// A^block = intermediate
///
pub struct SparseMatrix {
    l: u16,
    /// Indices of the source blocks which are xor-ed together
    /// | 0 0 1 1 |          [[ 2, 3],
    /// | 0 1 0 1 |           [ 1, 3 ],
    /// | 1 1 1 0 | -> coeff  [ 0, 1, 2],
    /// | 1 0 0 0 |           [ 0 ] ]
    ///
    /// M x L matrix
    a: Vec<SparseVector>,

    v_start_idx: u16,

    /// Intermediate symbol indices (size L)
    c: Vec<u16>,

    /// Encoding symbols (size M)
    D: Vec<Vec<u8>>,

    row_swaps: Vec<(u16, u16)>,
    col_swaps: Vec<(u16, u16)>,
}

impl SparseMatrix {
    pub fn new(l: u16) -> Self {
        Self {
            l,
            a: Vec::new(),
            v_start_idx: 0,
            c: (0..l).collect(),
            D: Vec::new(),

            row_swaps: Vec::new(),
            col_swaps: Vec::new(),
        }
    }

    fn swap_row(&mut self, first: u16, second: u16) {
        self.a.swap(first.into(), second.into());
        self.D.swap(first.into(), second.into());

        self.row_swaps.push((first, second));
    }

    fn swap_col(&mut self, from_start_row: u16, first: u16, second: u16) {
        for row in &mut self.a[from_start_row as usize..] {
            row.swap(first, second);
        }
        self.c.swap(first.into(), second.into());

        self.col_swaps.push((first, second));
    }

    /// * `components` - A vector of u32 numbers representing the indices of the intermediate blocks
    /// * `b` - A vector of u8 numbers representing the encoding symbol
    pub fn add_equation(&mut self, components: Vec<u16>, mut b: Vec<u8>) {
        // apply previous swaps to new equation
        let mut components = SparseVector::new(components);
        for (first, second) in self.col_swaps.iter().copied() {
            components.swap(first, second)
        }
        for (first, second) in self.row_swaps.iter().copied() {
            b.swap(first.into(), second.into());
        }

        // xor 0..self.v_start_idx rows into new row as necessary
        // we find the first index thats >= v_start_idx and drain 0..i
        let drain_until = match components.binary_search(&self.v_start_idx) {
            Ok(idx) => idx,
            Err(idx) => idx,
        };
        for zeroed_component in components.drain(0..drain_until) {
            // zeroed_component is equivalent to the row index that we need to xor
            common::xor_slice(&mut b, &self.D[zeroed_component as usize])
        }

        self.a.push(components);
        self.D.push(b);

        let inserted_components_idx = self.a.len() - 1;
        let inserted_components = &self.a[inserted_components_idx];
        let mut ready_to_peel = if inserted_components.len() == 1 {
            Some(inserted_components_idx)
        } else {
            None
        };
        while let Some(peel_idx) = ready_to_peel.take() {
            let peel_components = &self.a[peel_idx];
            self.swap_col(self.v_start_idx, self.v_start_idx, peel_components[0]);
            self.swap_row(self.v_start_idx, peel_idx as u16);
            self.v_start_idx += 1;

            for (row_idx, components) in
                (self.v_start_idx..).zip(self.a[self.v_start_idx as usize..].iter_mut())
            {
                let (d_first, d_second) = self.D.split_at_mut(row_idx.into());
                // xor 0..self.v_start_idx rows into new row as necessary
                // we find the first index thats >= v_start_idx and drain 0..i
                let drain_until = match components.binary_search(&self.v_start_idx) {
                    Ok(idx) => idx,
                    Err(idx) => idx,
                };
                // note that in this case, the loop can only execute at max once!
                for zeroed_component in components.drain(0..drain_until) {
                    // zeroed_component is equivalent to the row index that we need to xor
                    // note that zeroed_component is always < row_idx
                    common::xor_slice(&mut d_second[0], &d_first[zeroed_component as usize])
                }

                if components.len() == 1 {
                    ready_to_peel = Some(row_idx.into());
                }
            }
        }
    }

    /// Check is the decode matrix is fully specified
    pub fn fully_specified(&self) -> bool {
        self.v_start_idx == self.l
    }

    pub fn reduce(&mut self) {
        todo!()
    }

    pub fn intermediate_symbols(&self) -> Option<Vec<&Vec<u8>>> {
        if !self.fully_specified() {
            return None;
        }

        let intermediate_symbols = self.c.iter().map(|idx| &self.D[*idx as usize]).collect();
        Some(intermediate_symbols)
    }
}

// When we add an equation, if the row only has 1 coefficient in V (and nothing in U), then can
// immediately "peel" it. we can increment i and not increment u, in other words.
//
// U_upper is always zero in this cse. we immediately know the value of the intermediate symbols
// in question (until we get stuck)
//
// if the peeling never fails (unlikely), then V will eventually turn into identity matrix, with
// all-zeroes below it. we're done for free at this point! we discard the all-zero matrix under V
// and are left with an L x L identity matrix.
//
// if we do this "peeling" incrementally, then we also need to backtrack apply these changes to new
// equations that are inserted :)
//
// specifically, we need to backtrack the xor's that are done after the first row of V. the
// swapping of rows doesn't need to be backtracked because previous swaps will only ever be for
// previously existing rows.
// however, we do need to swap the components idx to match the intermediate symbol swaps that
// happened when columns were rearranged
//
// Q: how are the component swaps done efficiently in a sparse representation?
//
// maybe we need to be able to support both sparse and dense rows in the same matrix...
//
// Q: can we start building I from the bottom right instead? that would make it easier to trunate
// stuff out. but on other hand, that seems harder to do incrementally.
//
// --
//
// swapping A col j = swapping c row j
// swapping A row i = swapping d row i

#[cfg(test)]
mod tests {
    use crate::common;

    use super::SparseMatrix;

    fn validate_matrix(matrix: &SparseMatrix, symbols: &[Vec<u8>]) {
        assert!(matrix.fully_specified());
        let recovered_symbols: Vec<_> = matrix
            .intermediate_symbols()
            .unwrap()
            .into_iter()
            .take(symbols.len())
            .cloned()
            .collect();
        assert_eq!(symbols, recovered_symbols)
    }

    #[test]
    fn test_not_fully_specified() {
        let symbols = vec![vec![1, 1, 1, 1], vec![2, 2, 2, 2], vec![3, 3, 3, 3]];

        let mut matrix = SparseMatrix::new(3);
        matrix.add_equation(vec![0], symbols[0].clone());
        matrix.add_equation(vec![1], symbols[1].clone());
        assert!(!matrix.fully_specified());
    }

    #[test]
    fn test_fully_specified() {
        let symbols = vec![vec![1, 1, 1, 1], vec![2, 2, 2, 2], vec![3, 3, 3, 3]];
        let mut matrix = SparseMatrix::new(3);
        matrix.add_equation(vec![0], symbols[0].clone());
        matrix.add_equation(vec![1], symbols[1].clone());
        matrix.add_equation(vec![2], symbols[2].clone());

        validate_matrix(&matrix, &symbols);
    }

    #[test]
    fn test_fully_specified_rearranged() {
        let symbols = vec![vec![1, 1, 1, 1], vec![2, 2, 2, 2], vec![3, 3, 3, 3]];
        let mut matrix = SparseMatrix::new(3);
        matrix.add_equation(vec![0], symbols[0].clone());
        matrix.add_equation(vec![2], symbols[2].clone());
        matrix.add_equation(vec![1], symbols[1].clone());

        validate_matrix(&matrix, &symbols);
    }

    #[test]
    fn test_peeling() {
        let symbols = vec![vec![1, 1, 1, 1], vec![2, 2, 2, 2], vec![3, 3, 3, 3]];
        let mut matrix = SparseMatrix::new(3);

        let mut first_symbol = symbols[0].clone();
        common::xor_slice(&mut first_symbol, &symbols[1]);
        matrix.add_equation(vec![0, 1], first_symbol);
        matrix.add_equation(vec![2], symbols[2].clone());
        matrix.add_equation(vec![1], symbols[1].clone());

        validate_matrix(&matrix, &symbols);
    }
}
