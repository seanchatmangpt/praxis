//! Validate compliance gates

use std::path::Path;

use crate::{models::*, Result};

pub async fn validate_compliance(repo_path: &Path) -> Result<ComplianceReport> {
    let spec = crate::PraxisSpec::default();
    crate::audit::scan_repository(repo_path, &spec).await
}

pub fn is_fleet_compliant(reports: &[ComplianceReport]) -> bool {
    reports.iter().all(|r| r.is_compliant())
}

pub fn fleet_compliance_score(reports: &[ComplianceReport]) -> f32 {
    if reports.is_empty() {
        return 0.0;
    }
    let total_score: f32 = reports.iter().map(|r| r.score()).sum();
    total_score / reports.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_compliance_score_empty() {
        let reports = vec![];
        assert_eq!(fleet_compliance_score(&reports), 0.0);
    }
}
