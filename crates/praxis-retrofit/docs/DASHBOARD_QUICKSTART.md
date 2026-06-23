# Compliance Dashboard Quick Start

## 5-Minute Setup

### 1. Create Dashboard

```rust
use praxis_retrofit::compliance_dashboard::{Dashboard, DashboardConfig};

let config = DashboardConfig::default();
let mut dashboard = Dashboard::new(config);
```

### 2. Add Compliance Reports

```rust
// From your compliance audit
let report = audit::scan_repository(&repo_path, &spec)?;
dashboard.add_report(&report)?;
```

### 3. Get Fleet Status

```rust
let fleet = dashboard.get_fleet_status();
println!("Average compliance: {:.1}%", fleet.fleet_average_score);
println!("Passing repos: {}", fleet.passing_repos);
```

### 4. Check Alerts

```rust
for alert in dashboard.get_alerts() {
    println!("[{}] {}: {}", 
        alert.severity, 
        alert.repository, 
        alert.message);
}
```

### 5. Export Data

```rust
// JSON for external dashboards
let json = dashboard.export_json()?;

// Line protocol for time-series databases
let line_protocol = dashboard.export_line_protocol()?;
```

## Configuration

### Default Configuration

```rust
let config = DashboardConfig::default();
// alert_threshold: 80.0
// history_retention_days: 90
// enable_alerts: true
// dashboard_id: "praxis-compliance"
// category_weights: Optimized defaults
```

### Custom Configuration

```rust
let mut config = DashboardConfig::default();
config.alert_threshold = 85.0;        // Alert below 85%
config.history_retention_days = 30;   // Keep 30 days
config.enable_alerts = false;         // Disable alerts
config.dashboard_id = "my-fleet";     // Custom ID

let dashboard = Dashboard::new(config);
```

### Category Weights

Adjust how categories contribute to overall score:

```rust
config.category_weights.insert("supply-chain".to_string(), 1.5);  // Higher weight
config.category_weights.insert("editor-config".to_string(), 0.3); // Lower weight
```

## Common Operations

### Query Fleet Status

```rust
let fleet = dashboard.get_fleet_status();

// Overall metrics
fleet.fleet_average_score;      // 0-100
fleet.passing_repos;             // Count
fleet.failing_repos;             // Count
fleet.at_risk_repositories;      // Vec of repo names

// Category-level metrics
for (cat, metrics) in &fleet.fleet_category_summary {
    println!("{}: {:.1}%", cat, metrics.average_score);
    println!("  Pass rate: {:.1}%", metrics.pass_rate);
    println!("  Warnings: {}", metrics.repos_with_warnings);
    println!("  Failures: {}", metrics.repos_with_failures);
}
```

### Get Repository Status

```rust
if let Some(repo_status) = dashboard.repo_status.get("my-repo") {
    println!("Score: {:.1}%", repo_status.compliance_score);
    println!("Status: {:?}", repo_status.status);
    
    for (cat, cat_status) in &repo_status.category_status {
        println!("  {}: {:.1}%", cat, cat_status.score);
    }
}
```

### Get Trends

```rust
if let Some(trend) = dashboard.get_trend("my-repo") {
    println!("Direction: {}", trend.trend_direction);
    println!("Slope: {:.2}", trend.trend_slope);
    
    if let Some(days) = trend.days_to_alert {
        println!("Days to alert: {}", days);
    }
    
    for point in &trend.timeline {
        println!("  {} -> {:.1}%", point.timestamp, point.score);
    }
}
```

### Handle Alerts

```rust
// Get all active (unacknowledged) alerts
for alert in dashboard.get_alerts() {
    match alert.severity {
        AlertSeverity::Critical => {
            // Trigger escalation
            send_pagerduty_alert(alert);
        }
        AlertSeverity::Warning => {
            // Send Slack notification
            send_slack_warning(alert);
        }
        AlertSeverity::Info => {
            // Log for review
            log_info_alert(alert);
        }
    }
    
    // Acknowledge when handled
    dashboard.acknowledge_alert(&alert.alert_id);
}
```

### Record History

```rust
// Take a snapshot for historical tracking
dashboard.snapshot();

// Get historical data
let history = dashboard.get_history();
for status in history {
    println!("{}:  {:.1}%", status.timestamp, status.fleet_average_score);
}

// Clean old data (configurable retention)
dashboard.cleanup_old_history();
```

## Export Formats

### JSON Export

Full structured export for external systems:

```rust
let json = dashboard.export_json()?;
// Includes:
// - fleet_status: Current aggregated status
// - repositories: Per-repo details
// - trends: Historical trends
// - alerts: Active alerts
```

Use cases:
- Grafana dashboard ingestion
- Datadog metric import
- Custom dashboard creation
- Audit trail storage

### Line Protocol Export

Time-series database format (InfluxDB, Prometheus):

```rust
let line_protocol = dashboard.export_line_protocol()?;
// Format: metric_name,tags=values field=value timestamp
```

Use cases:
- Time-series storage
- Grafana datasource
- Prometheus scraping
- Long-term trend analysis

## Integration Patterns

### Pattern 1: Periodic Export (Recommended)

```rust
// Export to file every 5 minutes
loop {
    // Collect compliance reports
    for repo in get_repos() {
        let report = audit::scan_repository(&repo, &spec)?;
        dashboard.add_report(&report)?;
    }
    
    // Export
    let json = dashboard.export_json()?;
    fs::write("compliance.json", json)?;
    
    // Take snapshot
    dashboard.snapshot();
    
    // Clean old data
    dashboard.cleanup_old_history();
    
    // Wait
    std::thread::sleep(Duration::from_secs(300));
}
```

### Pattern 2: REST API Server

```rust
use actix_web::{web, App, HttpServer, HttpResponse};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dashboard = web::Data::new(
        std::sync::Mutex::new(Dashboard::new(DashboardConfig::default()))
    );
    
    HttpServer::new(move || {
        App::new()
            .app_data(dashboard.clone())
            .route("/api/fleet", web::get().to(|d: web::Data<_>| async move {
                let d = d.lock().unwrap();
                HttpResponse::Ok().json(d.get_fleet_status())
            }))
            .route("/api/export", web::get().to(|d: web::Data<_>| async move {
                let d = d.lock().unwrap();
                HttpResponse::Ok().json(d.export_json().unwrap())
            }))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

### Pattern 3: CI/CD Integration

```bash
#!/bin/bash
# Run in CI pipeline

# Scan all repos
praxis-retrofit audit scan --repos-file repos.txt \
  --output reports/

# Generate dashboard
cargo run --example dashboard_usage -- \
  --reports reports/ \
  --export dashboard.json

# Push to external system
curl -X POST http://dashboard.example.com/api/import \
  -H "Content-Type: application/json" \
  -d @dashboard.json
```

## Alert Handling

### Built-in Alert Types

1. **Threshold Breach**: Score drops below `alert_threshold`
2. **Sudden Drop**: Score decreases by >5% in one cycle
3. **Trending Down**: Negative trend slope

### Custom Alert Actions

```rust
for alert in dashboard.get_alerts() {
    match (&alert.severity, alert.repository.as_str()) {
        (AlertSeverity::Critical, "critical-repo") => {
            // Page on-call
            send_pagerduty(alert);
        }
        (AlertSeverity::Critical, _) => {
            // Slack critical channel
            send_slack_critical(alert);
        }
        (AlertSeverity::Warning, _) => {
            // Weekly digest
            add_to_digest(alert);
        }
        _ => {}
    }
}
```

## Performance Tips

1. **Batch Imports**: Add multiple reports before exporting
2. **Selective Export**: Export only when data changes
3. **History Cleanup**: Regular cleanup prevents memory bloat
4. **Snapshot Strategy**: Take snapshots at regular intervals, not on every update

```rust
let mut counter = 0;
for report in reports {
    dashboard.add_report(&report)?;
    counter += 1;
    
    // Export every 10 reports
    if counter % 10 == 0 {
        let json = dashboard.export_json()?;
        fs::write("compliance.json", json)?;
    }
}
```

## Troubleshooting

### No data showing up

```rust
// Check dashboard state
let fleet = dashboard.get_fleet_status();
if fleet.total_repos == 0 {
    println!("No repositories in dashboard");
    // Make sure to call add_report()
}
```

### Alerts not triggering

```rust
// Verify configuration
println!("Alert enabled: {}", dashboard.config.enable_alerts);
println!("Threshold: {}", dashboard.config.alert_threshold);

// Check repository scores
for repo in &dashboard.repo_status {
    if repo.1.compliance_score < dashboard.config.alert_threshold {
        println!("Repository {} should trigger alert", repo.0);
    }
}
```

### Missing historical data

```rust
// Check retention policy
println!("Retention: {} days", dashboard.config.history_retention_days);

// Manually cleanup if needed
dashboard.cleanup_old_history();
```

## Next Steps

1. **Integration**: Connect to Grafana/Datadog (see [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md))
2. **Customization**: Add custom alert handlers
3. **Automation**: Schedule dashboard exports via CI/CD
4. **Monitoring**: Set up external dashboards for team visibility

## API Reference

See full API docs: `cargo doc --open`

Key types:
- `Dashboard`: Main dashboard instance
- `DashboardConfig`: Configuration
- `FleetStatus`: Aggregated fleet metrics
- `RepositoryStatus`: Per-repo status
- `ComplianceAlert`: Alert notification
- `ComplianceTrend`: Historical trend data

## Examples

Run the example:

```bash
cargo run --example dashboard_usage
```

Expected output:
```
=== Praxis Compliance Dashboard Example ===

Step 1: Creating dashboard with configuration...
Dashboard created: example-fleet

Step 2: Adding compliance reports...
  Added: core-lib (score: 95.0%)
  Added: utils-kit (score: 88.0%)
  ...

Step 3: Fleet-wide status:
  Fleet Average: 87.3%
  Fleet Min:     75.0%
  Fleet Max:     100.0%
  Passing:       7
  Warnings:      2
  Failing:       1
  Total:         10

Step 4: At-risk repositories:
  - api-server: 75.0%
  - web-frontend: 80.0%

...
```
