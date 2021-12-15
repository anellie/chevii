const CAPACITY: usize = usize::pow(2, 18);
const MASK: usize = CAPACITY - 1;

/// Transmutation table.
/// Can be used multi-threaded; however it does not handle hash collisions or proper synchronization.
/// This is "safe".
pub struct TransTable {
    entries: Vec<Option<Entry>>
}

impl TransTable {
    pub fn get(&self, zobrist: u64) -> &Option<Entry> {
        let index = zobrist as usize & MASK;
        unsafe {
            match self.entries.get_unchecked(index) {
                Some(entry) if entry.zobrist == zobrist => self.entries.get_unchecked(index),
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

    pub fn new() -> Self {
        Self {
            entries: vec![None; CAPACITY]
        }
    }
}

#[derive(Clone)]
pub struct Entry {
    pub zobrist: u64,
    pub score: i32,
    pub depth: i32
}