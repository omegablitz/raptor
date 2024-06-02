use crate::common;

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
    a: Vec<Vec<u16>>,

    /// Intermediate symbol indices
    c: Vec<u16>,
    // /// Encoding symbol indices
    // d: Vec<u16>,
    /// Encoding symbols
    D: Vec<Vec<u8>>,
}

impl SparseMatrix {
    pub fn new(l: u16) -> Self {
        Self {
            l,
            a: Vec::new(),
            c: (0..l).collect(),
            // d: Vec::new(),
            D: Vec::new(),
        }
    }

    /// * `components` - A vector of u32 numbers representing the indices of the intermediate blocks
    /// * `b` - A vector of u8 numbers representing the encoding symbol
    pub fn add_equation(&mut self, components: Vec<u16>, b: Vec<u8>) {
        self.a.push(components);
        // self.d.push(
        //     // TODO fail decoding instead of panic?
        //     self.d
        //         .len()
        //         .try_into()
        //         .expect("# of encoded symbols exceeds u16 max"),
        // );
        self.D.push(b);
    }

    /// Check is the decode matrix is fully specified
    pub fn fully_specified(&self) -> bool {
        todo!()
    }

    pub fn reduce(&mut self) {
        todo!()
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
