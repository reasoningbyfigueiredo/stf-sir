use super::report::BenchmarkReport;

#[allow(dead_code)]
pub struct DriftDetector {
    pub threshold: f64,
}

#[derive(Debug, Clone)]
pub struct ComponentDrift {
    pub component: String,
    pub baseline: f64,
    pub current: f64,
    pub delta: f64,
    pub drift_detected: bool,
}

#[derive(Debug, Clone)]
pub struct DriftReport {
    pub detected: bool,
    pub component_drifts: Vec<ComponentDrift>,
}

impl DriftDetector {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Detect drift between a baseline report and a current report.
    ///
    /// Drift is detected for a component when `current - baseline < -threshold`
    /// (i.e., a regression of more than `threshold`).
    pub fn detect(&self, baseline: &BenchmarkReport, current: &BenchmarkReport) -> DriftReport {
        let pairs: &[(&str, f64, f64)] = &[
            ("rho_l", baseline.retention_v2.rho_l, current.retention_v2.rho_l),
            ("rho_s", baseline.retention_v2.rho_s, current.retention_v2.rho_s),
            (
                "rho_sigma_gloss",
                baseline.retention_v2.rho_sigma_gloss,
                current.retention_v2.rho_sigma_gloss,
            ),
            (
                "rho_sigma_concepts",
                baseline.retention_v2.rho_sigma_concepts,
                current.retention_v2.rho_sigma_concepts,
            ),
            ("rho_phi", baseline.retention_v2.rho_phi, current.retention_v2.rho_phi),
            (
                "rho_corpus",
                baseline.retention_v2.rho_corpus,
                current.retention_v2.rho_corpus,
            ),
        ];

        let component_drifts: Vec<ComponentDrift> = pairs
            .iter()
            .map(|(name, base_val, curr_val)| {
                let delta = curr_val - base_val;
                ComponentDrift {
                    component: name.to_string(),
                    baseline: *base_val,
                    current: *curr_val,
                    delta,
                    drift_detected: delta < -self.threshold,
                }
            })
            .collect();

        let detected = component_drifts.iter().any(|c| c.drift_detected);

        DriftReport { detected, component_drifts }
    }
}
