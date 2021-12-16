use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

const EMPTY: AtomicU32 = AtomicU32::new(0);
static STATS: [AtomicU32; 5] = [EMPTY; 5];
static STATS_LAST_DEPTH: [AtomicU32; 5] = [EMPTY; 5];
static STATS_THIS_DEPTH: [AtomicU32; 5] = [EMPTY; 5];

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
        STATS_THIS_DEPTH[*self as usize].fetch_add(1, Ordering::Relaxed);
    }

    pub fn next_depth() {
        for (this, last) in STATS_THIS_DEPTH.iter().zip(STATS_LAST_DEPTH.iter()) {
            last.store(this.load(Ordering::Relaxed), Ordering::Relaxed);
            this.store(0, Ordering::Relaxed);
        }
    }

    pub fn log() {
        log::debug!("Eval finished. Statistics for all depths:");
        Self::log_stats(&STATS);
        log::debug!("Eval finished. Statistics for final depth:");
        Self::log_stats(&STATS_LAST_DEPTH);
    }

    fn log_stats(stat: &[AtomicU32]) {
        log::debug!("   Nodes evaluated: {}", stat[0].load(Ordering::Relaxed));
        log::debug!(
            "   Transposition table hits: {}",
            stat[1].load(Ordering::Relaxed)
        );
        log::debug!(
            "   Transposition table misses: {}",
            stat[2].load(Ordering::Relaxed)
        );
        log::debug!("   Checkmates found: {}", stat[3].load(Ordering::Relaxed));
        log::debug!("   Branches pruned: {}", stat[4].load(Ordering::Relaxed));
    }
}
