const CAPACITY: usize = usize::pow(2, 18);
const MASK: usize = CAPACITY - 1;

/// Transmutation table.
/// Can be used multi-threaded; however it does not handle hash collisions or proper synchronization.
/// This is "safe".
pub struct TransTable {
    entries: Vec<Option<Entry>>,
    entries_nnue: Vec<Option<NNUEEntry>>,
}

impl TransTable {
    pub fn get(&self, zobrist: u64) -> &Option<Entry> {
        let index = zobrist as usize & MASK;
        unsafe {
            let val = self.entries.get_unchecked(index);
            match val {
                Some(entry) if entry.zobrist == zobrist => val,
                _ => &None,
            }
        }
    }

    pub fn get_nnue(&self, zobrist: u64) -> &Option<NNUEEntry> {
        let index = zobrist as usize & MASK;
        unsafe {
            let val = self.entries_nnue.get_unchecked(index);
            match val {
                Some(entry) if entry.zobrist == zobrist => val,
                _ => &None,
            }
        }
    }

    pub fn put(&self, entry: Entry) {
        let index = entry.zobrist as usize & MASK;
        unsafe {
            // Messing up safety one cast at a time!
            let vec = &self.entries as *const Vec<Option<Entry>>;
            let vecp = vec as *mut Vec<Option<Entry>>;
            let vecmut = &mut *vecp;
            vecmut[index] = Some(entry);
        }
    }

    pub fn put_nnue(&self, entry: NNUEEntry) {
        let index = entry.zobrist as usize & MASK;
        unsafe {
            // Messing up safety one cast at a time!
            let vec = &self.entries_nnue as *const Vec<Option<NNUEEntry>>;
            let vecp = vec as *mut Vec<Option<NNUEEntry>>;
            let vecmut = &mut *vecp;
            vecmut[index] = Some(entry);
        }
    }

    pub fn new() -> Self {
        Self {
            entries: vec![None; CAPACITY],
            entries_nnue: vec![None; CAPACITY],
        }
    }
}

#[derive(Clone)]
pub struct Entry {
    pub zobrist: u64,
    pub score: i32,
    pub depth_of_score: i16,
    pub depth_of_search: i16,
}

#[derive(Clone)]
pub struct NNUEEntry {
    pub zobrist: u64,
    pub score: i32,
}
