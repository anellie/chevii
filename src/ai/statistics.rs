use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

const EMPTY: AtomicU32 = AtomicU32::new(0);
static STATS: [AtomicU32; 5] = [EMPTY; 5];

#[derive(Copy, Clone)]
pub enum Stat {
    NodesEvaluated = 0,
    TableHits = 1,
    TableMisses = 2,
    CheckmatesFound = 3,
    BranchesCut = 4
}

impl Stat {
    pub fn inc(&self) {
        STATS[*self as usize].fetch_add(1, Ordering::Relaxed);
    }

    pub fn clear() {
        for stat in &STATS {
            stat.store(0, Ordering::Relaxed);
        }
    }

    pub fn log() {
        log::debug!("Eval finished. Statistics:");
        log::debug!("   Nodes evaluated: {}", STATS[0].load(Ordering::Relaxed));
        log::debug!(
            "   Transposition table hits: {}",
            STATS[1].load(Ordering::Relaxed)
        );
        log::debug!(
            "   Transposition table misses: {}",
            STATS[2].load(Ordering::Relaxed)
        );
        log::debug!("   Checkmates found: {}", STATS[3].load(Ordering::Relaxed));
        log::debug!("   Branches pruned: {}", STATS[4].load(Ordering::Relaxed));
        Self::clear();
    }
}
