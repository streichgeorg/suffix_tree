#[derive(Clone)]
pub struct Alphabet<'a> {
    pub size: u8,
    pub symbols: &'a [u8],
    ranks: [Option<u8>; 255],
}

impl<'a> Alphabet<'a> {
    pub fn new(symbols: &'a [u8]) -> Alphabet<'a> {
        let mut ranks = [None; 255];
        for (i, &symbol) in symbols.iter().enumerate() {
            assert!(ranks[symbol as usize].is_none(), "symbol appears twice in alphabet");
            ranks[symbol as usize] = Some(i as u8);
        }

        Alphabet {
            size: symbols.len() as u8,
            symbols,
            ranks,
        }
    }

    pub fn rank_of_symbol(&self, symbol: u8) -> u8 {
        self.ranks[symbol as usize].unwrap()
    }

    pub fn symbol_of_rank(&self, rank: u8) -> u8 {
        self.symbols[rank as usize]
    }
}

lazy_static! {
    pub static ref ASCII_LOWERCASE: Alphabet<'static> = Alphabet::new(b"abcdefghijklmnopqrstuvwxyz");
    pub static ref ASCII_UPPERCASE: Alphabet<'static> = Alphabet::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    pub static ref ASCII: Alphabet<'static> =
        Alphabet::new(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
}
