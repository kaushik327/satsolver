#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolarityHeuristic {
    AlwaysFalse,
    AlwaysTrue,
    PhaseSaving,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RestartStrategy {
    None,
    Luby { unit: u32 },
    Geometric { initial: u32, factor: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeletionStrategy {
    None,
    Lbd { max_lbd: u32 },
    Activity { fraction: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolverConfig {
    pub polarity: PolarityHeuristic,
    pub restart: RestartStrategy,
    pub deletion: DeletionStrategy,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            polarity: PolarityHeuristic::PhaseSaving,
            restart: RestartStrategy::Luby { unit: 100 },
            deletion: DeletionStrategy::Lbd { max_lbd: 6 },
        }
    }
}

// Luby sequence: luby(i) for i = 0, 1, 2, ...
// Two-counter recurrence: maintain (u, v); when v catches up to the lowest
// set bit of u (u & -u == v), reset v to 1 and advance u.
struct LubySequence {
    u: u32,
    v: u32,
}

impl LubySequence {
    fn new() -> Self {
        Self { u: 1, v: 1 }
    }

    fn next(&mut self) -> u32 {
        let val = self.v;
        if self.u & self.u.wrapping_neg() == self.v {
            self.u += 1;
            self.v = 1;
        } else {
            self.v *= 2;
        }
        val
    }
}

/// Tracks restart timing for a given strategy, advancing the threshold after each restart.
pub struct RestartScheduler {
    strategy: RestartStrategy,
    luby: LubySequence,
    next_restart: u32,
    geometric_gap: u32,
}

impl RestartScheduler {
    pub fn new(strategy: RestartStrategy) -> Self {
        let mut luby = LubySequence::new();
        let (next_restart, geometric_gap) = match strategy {
            RestartStrategy::None => (u32::MAX, 0),
            RestartStrategy::Luby { unit } => (luby.next() * unit, 0),
            RestartStrategy::Geometric { initial, .. } => (initial, initial),
        };
        Self {
            strategy,
            luby,
            next_restart,
            geometric_gap,
        }
    }

    pub fn should_restart(&self, conflict_count: u32) -> bool {
        conflict_count >= self.next_restart
    }

    /// Call immediately after performing a restart, passing the current conflict count.
    pub fn advance(&mut self, conflict_count: u32) {
        let gap = match self.strategy {
            RestartStrategy::None => u32::MAX,
            RestartStrategy::Luby { unit } => self.luby.next() * unit,
            RestartStrategy::Geometric { factor, .. } => {
                self.geometric_gap = ((self.geometric_gap as f64) * factor).ceil() as u32;
                self.geometric_gap
            }
        };
        self.next_restart = conflict_count + gap;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luby_sequence() {
        // 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8, ...
        let expected = [1u32, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8];
        let mut luby = LubySequence::new();
        for &exp in &expected {
            assert_eq!(luby.next(), exp);
        }
    }

    #[test]
    fn test_restart_scheduler_none_never_fires() {
        let sched = RestartScheduler::new(RestartStrategy::None);
        assert!(!sched.should_restart(0));
        assert!(!sched.should_restart(u32::MAX - 1));
    }

    #[test]
    fn test_restart_scheduler_luby_thresholds() {
        let mut sched = RestartScheduler::new(RestartStrategy::Luby { unit: 10 });
        // luby(0)=1 → first threshold at 10
        assert!(!sched.should_restart(9));
        assert!(sched.should_restart(10));
        sched.advance(10);
        // luby(1)=1 → next at 10+10=20
        assert!(!sched.should_restart(19));
        assert!(sched.should_restart(20));
        sched.advance(20);
        // luby(2)=2 → next at 20+20=40
        assert!(!sched.should_restart(39));
        assert!(sched.should_restart(40));
    }

    #[test]
    fn test_restart_scheduler_geometric_thresholds() {
        let mut sched = RestartScheduler::new(RestartStrategy::Geometric {
            initial: 10,
            factor: 2.0,
        });
        assert!(!sched.should_restart(9));
        assert!(sched.should_restart(10));
        sched.advance(10);
        // gap = ceil(10 * 2.0) = 20, next at 30
        assert!(!sched.should_restart(29));
        assert!(sched.should_restart(30));
        sched.advance(30);
        // gap = ceil(20 * 2.0) = 40, next at 70
        assert!(!sched.should_restart(69));
        assert!(sched.should_restart(70));
    }
}
