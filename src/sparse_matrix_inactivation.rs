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
    a: Vec<Vec<u16>>,

    // transposed sparse A
    // in other words, for each col_physical_idx, all physical rows
    a_t: Vec<Vec<u16>>,

    /// a_col_physical_to_virtual[1] = 0 means that a[i][1] is the first column
    a_col_physical_to_virtual: Vec<u16>,
    /// a_col_virtual_to_physical[0] = 1 means that the first column is at a[i][1]
    a_col_virtual_to_physical: Vec<u16>,

    // a_row_physical_to_virtual[1] = 0 means that a[1] is the first row
    a_row_physical_to_virtual: Vec<u16>,
    // a_row_virtual_to_physical[0] = 1 means that the first row is at a[1]
    a_row_virtual_to_physical: Vec<u16>,

    v_start_idx: u16,

    /// Intermediate symbol indices (size L)
    c: Vec<u16>,

    /// Encoding symbols (size M)
    D: Vec<Vec<u8>>,

    // FIXME hack
    max_symbol_size: usize,
}

impl SparseMatrix {
    pub fn new(l: u16) -> Self {
        Self {
            l,

            a: Vec::new(),
            a_t: vec![Vec::new(); l.into()],
            a_col_physical_to_virtual: (0..l).collect(),
            a_col_virtual_to_physical: (0..l).collect(),
            a_row_physical_to_virtual: Vec::new(),
            a_row_virtual_to_physical: Vec::new(),

            v_start_idx: 0,
            c: (0..l).collect(),
            D: Vec::new(),

            max_symbol_size: 0,
        }
    }

    fn swap_row(&mut self, first_virtual: u16, second_virtual: u16) {
        let (first_physical, second_physical) = (
            self.a_row_virtual_to_physical[first_virtual as usize],
            self.a_row_virtual_to_physical[second_virtual as usize],
        );
        self.a_row_physical_to_virtual
            .swap(first_physical.into(), second_physical.into());
        self.a_row_virtual_to_physical
            .swap(first_virtual.into(), second_virtual.into());

        assert_eq!(
            self.a_row_virtual_to_physical
                [self.a_row_physical_to_virtual[first_physical as usize] as usize],
            first_physical
        );
        assert_eq!(
            self.a_row_virtual_to_physical
                [self.a_row_physical_to_virtual[second_physical as usize] as usize],
            second_physical
        );
        assert_eq!(
            self.a_row_physical_to_virtual
                [self.a_row_virtual_to_physical[first_virtual as usize] as usize],
            first_virtual
        );
        assert_eq!(
            self.a_row_physical_to_virtual
                [self.a_row_virtual_to_physical[first_virtual as usize] as usize],
            first_virtual
        );

        self.D.swap(first_virtual.into(), second_virtual.into());
    }

    fn swap_col(&mut self, first_virtual: u16, second_virtual: u16) {
        let (first_physical, second_physical) = (
            self.a_col_virtual_to_physical[first_virtual as usize],
            self.a_col_virtual_to_physical[second_virtual as usize],
        );
        self.a_col_physical_to_virtual
            .swap(first_physical.into(), second_physical.into());
        self.a_col_virtual_to_physical
            .swap(first_virtual.into(), second_virtual.into());

        assert_eq!(
            self.a_col_virtual_to_physical
                [self.a_col_physical_to_virtual[first_physical as usize] as usize],
            first_physical
        );
        assert_eq!(
            self.a_col_virtual_to_physical
                [self.a_col_physical_to_virtual[second_physical as usize] as usize],
            second_physical
        );
        assert_eq!(
            self.a_col_physical_to_virtual
                [self.a_col_virtual_to_physical[first_virtual as usize] as usize],
            first_virtual
        );
        assert_eq!(
            self.a_col_physical_to_virtual
                [self.a_col_virtual_to_physical[first_virtual as usize] as usize],
            first_virtual
        );

        self.c.swap(first_virtual.into(), second_virtual.into());
    }

    /// * `components` - A vector of u32 numbers representing the indices of the intermediate blocks
    /// * `b` - A vector of u8 numbers representing the encoding symbol
    pub fn add_equation(&mut self, mut components: Vec<u16>, mut b: Vec<u8>) {
        if self.max_symbol_size < b.len() {
            self.max_symbol_size = b.len();
            for symbol in &mut self.D {
                symbol.resize(self.max_symbol_size, 0);
            }
        }
        b.resize(self.max_symbol_size, 0);

        // xor 0..self.v_start_idx rows into new row as necessary
        components.retain({
            |physical_col_idx| {
                let virtual_col_idx = self.a_col_physical_to_virtual[*physical_col_idx as usize];
                let retain = virtual_col_idx >= self.v_start_idx;
                if !retain {
                    // virtual_col_idx is equivalent to the row index that we need to xor
                    common::xor_slice(&mut b, &self.D[virtual_col_idx as usize])
                }
                retain
            }
        });

        self.a_row_physical_to_virtual.push(self.a.len() as u16);
        self.a_row_virtual_to_physical.push(self.a.len() as u16);
        for component in &components {
            self.a_t[*component as usize].push(self.a.len() as u16);
        }
        let mut try_peel = vec![self.a.len()];
        self.a.push(components);

        self.D.push(b);

        while let Some(physical_peel_idx) = try_peel.pop() {
            let peel_components = &self.a[physical_peel_idx];
            let virtual_peel_idx = self.a_row_physical_to_virtual[physical_peel_idx];
            if virtual_peel_idx < self.v_start_idx || peel_components.len() != 1 {
                // has been already peeled, or can't peel
                continue;
            }
            self.swap_col(
                self.v_start_idx,
                self.a_col_physical_to_virtual[peel_components[0] as usize],
            );
            self.swap_row(self.v_start_idx, virtual_peel_idx);

            let physical_v_start_row = self.a_row_virtual_to_physical[self.v_start_idx as usize];
            let physical_v_start_col = self.a_col_virtual_to_physical[self.v_start_idx as usize];

            self.a_t[physical_v_start_col as usize].retain(|physical_row_idx| {
                let retain = physical_row_idx == &physical_v_start_row;
                if !retain {
                    let (d_first, d_second) = self.D.split_at_mut(
                        self.a_row_physical_to_virtual[*physical_row_idx as usize].into(),
                    );

                    let a_row = &mut self.a[*physical_row_idx as usize];
                    let idx = a_row
                        .binary_search(&physical_v_start_col)
                        .expect("if exists in a_t, must exist in a");
                    a_row.remove(idx);
                    common::xor_slice(&mut d_second[0], &d_first[self.v_start_idx as usize]);
                    if a_row.len() == 1 {
                        try_peel.push((*physical_row_idx).into());
                    }
                }
                retain
            });

            self.v_start_idx += 1;
        }
    }

    /// Check is the decode matrix is fully specified
    pub fn fully_specified(&self) -> bool {
        self.v_start_idx == self.l
    }

    pub fn reduce(&mut self) {
        assert!(self.fully_specified());
    }

    pub fn intermediate_symbols(&self) -> Option<Vec<&Vec<u8>>> {
        if !self.fully_specified() {
            return None;
        }
        let mut intermediate_symbols = vec![&self.D[0]; self.c.len()];
        for (c, d) in self.c.iter().zip(&self.D) {
            intermediate_symbols[*c as usize] = d;
        }
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
        let symbols = vec![vec![1, 2, 3, 4], vec![2, 3, 4, 5], vec![3, 4, 5, 6]];

        let mut matrix = SparseMatrix::new(3);
        matrix.add_equation(vec![0], symbols[0].clone());
        matrix.add_equation(vec![1], symbols[1].clone());
        assert!(!matrix.fully_specified());
    }

    #[test]
    fn test_fully_specified() {
        let symbols = vec![vec![1, 2, 3, 4], vec![2, 3, 4, 5], vec![3, 4, 5, 6]];
        let mut matrix = SparseMatrix::new(3);
        matrix.add_equation(vec![0], symbols[0].clone());
        matrix.add_equation(vec![1], symbols[1].clone());
        matrix.add_equation(vec![2], symbols[2].clone());

        validate_matrix(&matrix, &symbols);
    }

    #[test]
    fn test_fully_specified_rearranged() {
        let symbols = vec![vec![1, 2, 3, 4], vec![2, 3, 4, 5], vec![3, 4, 5, 6]];
        let mut matrix = SparseMatrix::new(3);
        matrix.add_equation(vec![0], symbols[0].clone());
        matrix.add_equation(vec![2], symbols[2].clone());
        matrix.add_equation(vec![1], symbols[1].clone());

        validate_matrix(&matrix, &symbols);
    }

    #[test]
    fn test_peeling() {
        let symbols = vec![vec![1, 2, 3, 4], vec![2, 3, 4, 5], vec![3, 4, 5, 6]];
        let mut matrix = SparseMatrix::new(3);

        let mut second_symbol = symbols[0].clone();
        common::xor_slice(&mut second_symbol, &symbols[1]);
        common::xor_slice(&mut second_symbol, &symbols[2]);
        matrix.add_equation(vec![0, 1, 2], second_symbol);

        let mut first_symbol = symbols[0].clone();
        common::xor_slice(&mut first_symbol, &symbols[2]);
        matrix.add_equation(vec![0, 2], first_symbol);

        matrix.add_equation(vec![2], symbols[2].clone());

        validate_matrix(&matrix, &symbols);
    }

    #[test]
    fn test_extra() {
        let symbols = vec![vec![1, 2, 3, 4], vec![2, 3, 4, 5], vec![3, 4, 5, 6]];
        let mut matrix = SparseMatrix::new(3);

        matrix.add_equation(vec![0], symbols[0].clone());
        matrix.add_equation(vec![1], symbols[1].clone());
        matrix.add_equation(vec![0], symbols[0].clone());
        matrix.add_equation(vec![1], symbols[1].clone());
        matrix.add_equation(vec![0], symbols[0].clone());
        matrix.add_equation(vec![1], symbols[1].clone());
        let mut first_symbol = symbols[0].clone();
        common::xor_slice(&mut first_symbol, &symbols[2]);
        matrix.add_equation(vec![0, 2], first_symbol);

        validate_matrix(&matrix, &symbols);
    }

    #[test]
    fn test_zero() {
        let symbols = vec![vec![1, 2, 3, 4], vec![2, 3, 4, 5], vec![0, 0, 0, 0]];
        let mut matrix = SparseMatrix::new(3);

        let mut first_symbol = symbols[0].clone();
        common::xor_slice(&mut first_symbol, &symbols[1]);
        matrix.add_equation(vec![0, 1], first_symbol);
        matrix.add_equation(vec![2], vec![]);
        matrix.add_equation(vec![1], symbols[1].clone());

        validate_matrix(&matrix, &symbols);
    }
}
