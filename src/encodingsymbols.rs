use bytes::Bytes;

pub struct EncodingSymbol {
    pub data: Bytes,
    pub esi: u32,
}

impl EncodingSymbol {
    pub fn new(data: Bytes, esi: u32) -> Self {
        EncodingSymbol { data, esi }
    }

    pub fn from_option_block(block: &[Option<Vec<u8>>]) -> Vec<EncodingSymbol> {
        block
            .iter()
            .enumerate()
            .filter(|(_, symbols)| symbols.is_some())
            .map(|(esi, symbols)| EncodingSymbol {
                data: Bytes::copy_from_slice(symbols.as_ref().unwrap()),
                esi: esi as u32,
            })
            .collect()
    }
}
