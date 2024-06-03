use crate::common;
use crate::partition::Partition;

/// A struct that represents a source block encoder that uses Raptor codes.
pub struct SourceBlockEncoder {
    intermediate: Vec<Vec<u8>>,
    k: u32,
    l: u32,
    l_prime: u32,
}

impl SourceBlockEncoder {
    /// Create a source block encoder, passing the list of source symbols
    ///
    /// # Parameters
    ///
    /// * `source_block`: A slice of vectors containing the source symbols.
    /// * `max_source_symbols`: Max number of source symbols inside the source block
    ///
    /// # Returns
    ///
    /// A new `SourceBlockEncoder` instance.
    pub fn new(source_block: &[u8], max_source_symbols: usize) -> Self {
        let partition = Partition::new(source_block.len(), max_source_symbols);
        let source_block = partition.create_source_block(source_block);
        let k = source_block.len() as u32;

        // let mut raptor = raptor::Raptor::new(k);
        // raptor.add_encoding_symbols(&source_block);
        // raptor.reduce();
        // let intermediate = raptor.intermediate_symbols().to_vec();

        let (l, l_prime, _s, _h, _hp) = common::intermediate_symbols(k);
        let intermediate = super::raptor::Raptor::create_intermediate_symbols(k, &source_block);

        SourceBlockEncoder {
            intermediate,
            k,
            l,
            l_prime,
        }
    }

    /// Return the number of source symbols (k) inside the block
    pub fn nb_source_symbols(&self) -> u32 {
        self.k
    }

    /// Generates an encoding symbol with the specified Encoding Symbol Identifier (ESI).
    ///
    /// This method generates a encoding symbol using the Raptor code and the intermediate symbols generated during the initialization of the encoder.
    ///
    /// # Parameters
    ///
    /// * `esi`: The Encoding Symbol Identifier (ESI) of the desired encoding symbol.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `Vec<u8>` : The generated encoding symbol
    pub fn fountain(&self, esi: u32) -> Vec<u8> {
        let mut block = Vec::new();
        let indices = common::find_lt_indices(self.k, esi, self.l, self.l_prime);
        for indice in indices {
            if indice < self.intermediate.len() as u16 {
                common::xor(&mut block, &self.intermediate[indice as usize]);
            }
        }

        block
    }

    /// returns length of generated chunks
    pub fn chunk_len(&self) -> usize {
        self.intermediate[0].len()
    }

    /// faster fountain?
    pub fn fountain2(&mut self, output: &mut [u8]) {
        let symbol_size = self.intermediate[0].len();
        for esi in 0..output.len() / symbol_size {
            let indices = common::find_lt_indices(self.k, esi as u32, self.l, self.l_prime);
            for indice in indices {
                if indice < self.intermediate.len() as u16 {
                    common::xor_slice(
                        &mut output[esi * symbol_size..(esi + 1) * symbol_size],
                        &self.intermediate[indice as usize],
                    );
                }
            }
        }
    }

    /// faster fountain?
    pub fn fountain3(&mut self, esi: u32, output: &mut [u8]) {
        assert_eq!(output.len(), self.chunk_len());
        let indices = common::find_lt_indices(self.k, esi, self.l, self.l_prime);
        for indice in indices {
            if indice < self.intermediate.len() as u16 {
                common::xor_slice(output, &self.intermediate[indice as usize]);
            }
        }
    }
}

///
/// Encodes a source block into encoding symbols using Raptor codes.
///
/// # Parameters
///
/// * `source_block`: A slice of vectors containing the source symbols.
/// * `max_source_symbols`: Max number of source symbols inside the source block
/// * `nb_repair`: The number of repair symbols to be generated.
///
/// # Returns
///
/// a Tuple
/// * `Vec<Vec<u8>>` : A vector of vectors of bytes representing the encoding symbols (source symbols + repair symbol).
/// * `u32` : Number of source symbols (k)
///
///
/// The function uses Raptor codes to generate the specified number of repair symbols from the source block.
///
pub fn encode_source_block(
    source_block: &[u8],
    max_source_symbols: usize,
    nb_repair: usize,
) -> (Vec<Vec<u8>>, u32) {
    let mut encoder = SourceBlockEncoder::new(source_block, max_source_symbols);
    let mut output: Vec<Vec<u8>> = Vec::new();
    let n = encoder.nb_source_symbols() as usize + nb_repair;
    for esi in 0..n as u32 {
        output.push(encoder.fountain(esi));
    }
    (output, encoder.nb_source_symbols())
}

/// faster?
pub fn encode_source_block_2(
    source_block: &[u8],
    max_source_symbols: usize,
    nb_repair: usize,
) -> (Vec<u8>, u32) {
    let mut encoder = SourceBlockEncoder::new(source_block, max_source_symbols);
    let n = encoder.nb_source_symbols() as usize + nb_repair;

    let mut output = vec![0_u8; n * encoder.intermediate[0].len()];
    encoder.fountain2(&mut output);
    (output, encoder.nb_source_symbols())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_source_block_encoder() {
        crate::tests::init();

        let input: Vec<u8> = vec![1, 2, 7, 4, 0, 2, 54, 4, 1, 1, 10, 200, 1, 21, 3, 80];
        let max_source_symbols = 4;
        let nb_repair = 3;

        let (encoded_block, k) = super::encode_source_block(&input, max_source_symbols, nb_repair);
        log::debug!("Encoded with {} blocks", k);

        // Try to decode the source block

        let mut encoded_block: Vec<Option<Vec<u8>>> = encoded_block
            .into_iter()
            .map(|symbols| Some(symbols))
            .collect();

        // Simulate loss
        encoded_block[0] = None;
        encoded_block[1] = None;

        let output =
            crate::decoder::decode_source_block(&encoded_block, k as usize, input.len()).unwrap();
        log::debug!("{:?} / {:?}", output, input);
        assert!(output.len() == input.len());
        assert!(output == input);
    }
}
