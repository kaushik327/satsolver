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
