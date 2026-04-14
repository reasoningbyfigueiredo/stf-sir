/// The three truth states for a coherence dimension.
///
/// - `Satisfied` — the dimension holds.
/// - `Violated`  — the dimension fails.
/// - `Unknown`   — not yet evaluated (e.g., computational coherence before
///   budget is applied).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TruthValue {
    Satisfied,
    Violated,
    Unknown,
}

impl TruthValue {
    pub fn is_satisfied(self) -> bool {
        matches!(self, TruthValue::Satisfied)
    }

    pub fn is_violated(self) -> bool {
        matches!(self, TruthValue::Violated)
    }
}

impl std::fmt::Display for TruthValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TruthValue::Satisfied => write!(f, "satisfied"),
            TruthValue::Violated => write!(f, "violated"),
            TruthValue::Unknown => write!(f, "unknown"),
        }
    }
}

/// The coherence triple Coh(S) = (C_l, C_c, C_o) ∈ {0,1}³.
///
/// | C_l | C_c | C_o | Classification                           |
/// |-----|-----|-----|------------------------------------------|
/// |  0  |  -  |  -  | Contradictory — invalid at base level    |
/// |  1  |  0  |  -  | Intractable — coherent but unverifiable  |
/// |  1  |  1  |  0  | Sterile — coherent but informationally inert |
/// |  1  |  1  |  1  | Full coherence — the only productive state |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoherenceVector {
    /// C_l: logical coherence (absence of contradiction).
    pub logical: TruthValue,
    /// C_c: computational coherence (verifiable in polynomial time).
    pub computational: TruthValue,
    /// C_o: operational coherence (produces non-trivial consequences).
    pub operational: TruthValue,
}

impl CoherenceVector {
    /// The fully coherent state (1, 1, 1).
    pub fn full() -> Self {
        Self {
            logical: TruthValue::Satisfied,
            computational: TruthValue::Satisfied,
            operational: TruthValue::Satisfied,
        }
    }

    /// A contradictory state (0, -, -).
    pub fn contradictory() -> Self {
        Self {
            logical: TruthValue::Violated,
            computational: TruthValue::Unknown,
            operational: TruthValue::Unknown,
        }
    }

    /// A sterile state (1, 1, 0): coherent but operationally inert.
    pub fn sterile() -> Self {
        Self {
            logical: TruthValue::Satisfied,
            computational: TruthValue::Satisfied,
            operational: TruthValue::Violated,
        }
    }

    /// Returns `true` only when all three dimensions are `Satisfied`.
    pub fn is_full(&self) -> bool {
        self.logical.is_satisfied()
            && self.computational.is_satisfied()
            && self.operational.is_satisfied()
    }

    /// Returns `true` if logical coherence is violated (contradiction).
    pub fn is_contradictory(&self) -> bool {
        self.logical.is_violated()
    }

    /// Human-readable label for the coherence state.
    pub fn label(&self) -> &'static str {
        match (&self.logical, &self.computational, &self.operational) {
            (TruthValue::Violated, _, _) => "contradictory",
            (TruthValue::Satisfied, TruthValue::Violated, _) => "intractable",
            (TruthValue::Satisfied, _, TruthValue::Violated) => "sterile",
            (TruthValue::Satisfied, TruthValue::Satisfied, TruthValue::Satisfied) => "full",
            _ => "partial",
        }
    }
}

impl std::fmt::Display for CoherenceVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {}) [{}]",
            self.logical,
            self.computational,
            self.operational,
            self.label()
        )
    }
}
