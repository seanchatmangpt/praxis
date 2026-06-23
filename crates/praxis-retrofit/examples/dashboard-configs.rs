//! Example dashboard configurations for different deployment scenarios

use praxis_retrofit::compliance_dashboard::DashboardConfig;
use std::collections::HashMap;

/// Configuration for strict compliance monitoring
/// Use when compliance is critical (security-sensitive repos)
pub fn strict_config() -> DashboardConfig {
    let mut config = DashboardConfig::default();

    // Very low alert threshold
    config.alert_threshold = 90.0;  // Alert if below 90%
    config.enable_alerts = true;
    config.dashboard_id = "strict-compliance".to_string();

    // High weights for critical categories
    config.category_weights.insert("ci-cd".to_string(), 1.5);
    config.category_weights.insert("supply-chain".to_string(), 1.5);
    config.category_weights.insert("licensing".to_string(), 1.3);

    // Lower weights for less critical areas
    config.category_weights.insert("editor-config".to_string(), 0.3);
    config.category_weights.insert("versioning".to_string(), 0.3);

    config
}

/// Configuration for development/test environments
/// Use for early-stage projects and internal experiments
pub fn dev_config() -> DashboardConfig {
    let mut config = DashboardConfig::default();

    // Lenient thresholds
    config.alert_threshold = 70.0;  // Alert if below 70%
    config.enable_alerts = true;
    config.dashboard_id = "dev-compliance".to_string();

    // Shorter history retention for faster feedback
    config.history_retention_days = 14;

    // Balanced weights
    for (_, weight) in config.category_weights.iter_mut() {
        *weight = 1.0;
    }

    config
}

/// Configuration for production fleet monitoring
/// Use for mature, well-established projects
pub fn production_config() -> DashboardConfig {
    let mut config = DashboardConfig::default();

    // Standard alert threshold
    config.alert_threshold = 85.0;  // Alert if below 85%
    config.enable_alerts = true;
    config.dashboard_id = "prod-compliance".to_string();

    // Longer history retention for trend analysis
    config.history_retention_days = 90;

    // Balanced weights with slight emphasis on supply chain
    config.category_weights.insert("supply-chain".to_string(), 1.1);
    config.category_weights.insert("ci-cd".to_string(), 1.1);
    config.category_weights.insert("licensing".to_string(), 1.0);
    config.category_weights.insert("linting".to_string(), 0.9);
    config.category_weights.insert("documentation".to_string(), 0.8);
    config.category_weights.insert("editor-config".to_string(), 0.4);
    config.category_weights.insert("versioning".to_string(), 0.5);

    config
}

/// Configuration for analytics-heavy deployments
/// Use when you need detailed trend analysis and historical data
pub fn analytics_config() -> DashboardConfig {
    let mut config = DashboardConfig::default();

    config.alert_threshold = 80.0;
    config.enable_alerts = true;
    config.dashboard_id = "analytics-compliance".to_string();

    // Long retention for deep analysis
    config.history_retention_days = 365;  // 1 year

    // Equal weights for all categories to see unbiased trends
    for (_, weight) in config.category_weights.iter_mut() {
        *weight = 1.0;
    }

    config
}

/// Configuration with disabled alerts
/// Use when you want monitoring without notifications
pub fn silent_config() -> DashboardConfig {
    let mut config = DashboardConfig::default();

    config.alert_threshold = 80.0;
    config.enable_alerts = false;  // No automatic alerts
    config.dashboard_id = "silent-compliance".to_string();
    config.history_retention_days = 30;

    config
}

/// Configuration focused on supply chain security
/// Use for security-sensitive or regulated environments
pub fn security_focused_config() -> DashboardConfig {
    let mut config = DashboardConfig::default();

    config.alert_threshold = 90.0;
    config.enable_alerts = true;
    config.dashboard_id = "security-compliance".to_string();

    // Heavy emphasis on security-related categories
    config.category_weights.clear();
    config.category_weights.insert("supply-chain".to_string(), 2.0);  // Critical
    config.category_weights.insert("ci-cd".to_string(), 2.0);        // Critical
    config.category_weights.insert("licensing".to_string(), 1.5);    // Important
    config.category_weights.insert("linting".to_string(), 1.0);      // Moderate
    config.category_weights.insert("documentation".to_string(), 0.8); // Nice to have
    config.category_weights.insert("editor-config".to_string(), 0.2); // Minor
    config.category_weights.insert("versioning".to_string(), 0.5);   // Low

    // Keep 1 year for security audit trails
    config.history_retention_days = 365;

    config
}

/// Configuration for compliance report generation
/// Use when building reports for stakeholders
pub fn reporting_config() -> DashboardConfig {
    let mut config = DashboardConfig::default();

    config.alert_threshold = 85.0;
    config.enable_alerts = false;  // Reports don't need active alerts
    config.dashboard_id = "reporting-compliance".to_string();

    // Long retention for comprehensive reports
    config.history_retention_days = 180;  // 6 months

    // Balanced weights for fair assessment
    for (_, weight) in config.category_weights.iter_mut() {
        *weight = 1.0;
    }

    config
}

/// Configuration template for custom scenarios
pub fn custom_config(
    alert_threshold: f32,
    enable_alerts: bool,
    dashboard_id: &str,
    retention_days: i64,
    category_weights: HashMap<String, f32>,
) -> DashboardConfig {
    DashboardConfig {
        alert_threshold,
        enable_alerts,
        dashboard_id: dashboard_id.to_string(),
        history_retention_days: retention_days,
        category_weights,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_config() {
        let config = strict_config();
        assert_eq!(config.alert_threshold, 90.0);
        assert_eq!(config.dashboard_id, "strict-compliance");
        assert!(config.enable_alerts);
    }

    #[test]
    fn test_dev_config() {
        let config = dev_config();
        assert_eq!(config.alert_threshold, 70.0);
        assert_eq!(config.history_retention_days, 14);
    }

    #[test]
    fn test_production_config() {
        let config = production_config();
        assert_eq!(config.alert_threshold, 85.0);
        assert_eq!(config.history_retention_days, 90);
        assert!(config.enable_alerts);
    }

    #[test]
    fn test_security_focused_config() {
        let config = security_focused_config();
        assert!(config.alert_threshold >= 90.0);
        assert_eq!(config.history_retention_days, 365);

        // Supply chain should have highest weight
        assert_eq!(
            config.category_weights.get("supply-chain"),
            Some(&2.0)
        );
    }

    #[test]
    fn test_config_weights_sum() {
        let configs = vec![
            strict_config(),
            dev_config(),
            production_config(),
            security_focused_config(),
        ];

        for config in configs {
            // Weights should be positive
            for (_, weight) in &config.category_weights {
                assert!(*weight > 0.0);
            }

            // Should have all standard categories
            assert!(config.category_weights.contains_key("ci-cd"));
            assert!(config.category_weights.contains_key("supply-chain"));
        }
    }
}

fn main() {
    println!("=== Dashboard Configuration Examples ===\n");

    let configs = vec![
        ("Strict (Security-Critical)", strict_config()),
        ("Development", dev_config()),
        ("Production", production_config()),
        ("Analytics", analytics_config()),
        ("Silent (No Alerts)", silent_config()),
        ("Security-Focused", security_focused_config()),
        ("Reporting", reporting_config()),
    ];

    for (name, config) in configs {
        println!("{}:", name);
        println!("  Alert Threshold: {:.1}%", config.alert_threshold);
        println!("  Enable Alerts: {}", config.enable_alerts);
        println!("  History Retention: {} days", config.history_retention_days);
        println!("  Dashboard ID: {}", config.dashboard_id);
        println!("  Category Weights:");

        let categories = vec![
            "ci-cd",
            "supply-chain",
            "linting",
            "editor-config",
            "documentation",
            "licensing",
            "versioning",
        ];

        for cat in categories {
            if let Some(weight) = config.category_weights.get(cat) {
                println!("    {}: {:.1}x", cat, weight);
            }
        }
        println!();
    }
}
