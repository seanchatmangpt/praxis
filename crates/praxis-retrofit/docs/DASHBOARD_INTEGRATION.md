# Compliance Dashboard Integration Guide

This guide explains how to integrate the Praxis Compliance Dashboard with external monitoring systems like Grafana, Datadog, Prometheus, and Splunk.

## Table of Contents

1. [Overview](#overview)
2. [Dashboard Architecture](#dashboard-architecture)
3. [Integration Methods](#integration-methods)
4. [Grafana Integration](#grafana-integration)
5. [Datadog Integration](#datadog-integration)
6. [Prometheus Integration](#prometheus-integration)
7. [Splunk Integration](#splunk-integration)
8. [Alert Configuration](#alert-configuration)
9. [Custom Integrations](#custom-integrations)
10. [Troubleshooting](#troubleshooting)

## Overview

The Praxis Compliance Dashboard provides real-time monitoring of compliance status across 18+ repositories. It exports data in multiple formats:

- **JSON**: Complete export for manual ingestion or custom tooling
- **Line Protocol**: InfluxDB/Prometheus native format for time-series storage
- **REST API**: HTTP endpoints for polling

### Key Features

- Real-time compliance status across entire fleet
- Per-repository and per-category metrics
- Historical trend tracking (7-day moving average)
- Alert system with severity levels (Info/Warning/Critical)
- Automatic threshold detection and alerting
- Support for weighted category scoring

## Dashboard Architecture

### Core Components

```
┌─────────────────────────────────────────────────────┐
│          Praxis Retrofit (Repository Audits)        │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│     Compliance Dashboard (Fleet Aggregation)        │
│  - Repository Status Tracking                       │
│  - Trend Analysis (7-day moving average)            │
│  - Alert Generation                                 │
│  - Score Calculation (weighted by category)         │
└──────────────┬──────────────────┬───────────────────┘
               │                  │
        ┌──────▼──────┐    ┌──────▼──────────┐
        │ JSON Export │    │ Line Protocol   │
        └──────┬──────┘    └──────┬──────────┘
               │                  │
        ┌──────▼──────────────────▼──────┐
        │   External Monitoring Systems   │
        │ (Grafana, Datadog, Prometheus) │
        └─────────────────────────────────┘
```

### Data Flow

1. **Collection**: Repository audits generate `ComplianceReport`
2. **Aggregation**: Dashboard aggregates reports across fleet
3. **Calculation**: Scores, trends, and alerts computed
4. **Export**: Data formatted for external systems
5. **Ingestion**: External systems pull/push data
6. **Visualization**: Dashboards and alerts displayed

### Supported Metrics

#### Fleet-Level Metrics
- `fleet_average_score`: Average compliance (0-100%)
- `fleet_min_score`: Minimum compliance in fleet
- `fleet_max_score`: Maximum compliance in fleet
- `passing_repos`: Count of fully compliant repos
- `warning_repos`: Count of repos with warnings
- `failing_repos`: Count of non-compliant repos

#### Repository-Level Metrics
- `compliance_score`: Per-repo compliance (0-100%)
- `status`: Pass/Warn/Fail
- `critical_issues`: Number of failing checks
- `category_score`: Score per compliance category

#### Category-Level Metrics
- `average_score`: Category average across fleet
- `pass_rate`: Percentage of repos passing category
- `repos_with_warnings`: Count of repos with warnings
- `repos_with_failures`: Count of repos failing

#### Trend Metrics
- `trend_direction`: Improving/Stable/Declining
- `trend_slope`: Score change per day
- `days_to_alert`: Days until threshold breach (if declining)

## Integration Methods

### Method 1: JSON Export (Pull-Based)

**Best for**: One-time ingestion, manual imports, webhook-triggered updates

```rust
use praxis_retrofit::compliance_dashboard::{Dashboard, DashboardConfig};
use std::fs;

// Load configuration
let config = DashboardConfig::default();
let mut dashboard = Dashboard::new(config);

// Add compliance reports
dashboard.add_report(&compliance_report)?;

// Export as JSON
let json_data = dashboard.export_json()?;

// Save to file for ingestion
fs::write("dashboard-export.json", json_data)?;
```

**File Format**: See [dashboard-schema.json](../examples/dashboard-schema.json)

### Method 2: Line Protocol Export (Time-Series)

**Best for**: InfluxDB, Prometheus, Datadog Agent

```rust
let line_protocol = dashboard.export_line_protocol()?;

// Push to InfluxDB
// POST /api/v1/write?db=compliance
// Content-Type: text/plain

// Or write to file
fs::write("dashboard.lp", line_protocol)?;
```

### Method 3: REST API (Continuous Polling)

**Best for**: Native integrations, real-time dashboards

Create a simple HTTP server to expose dashboard data:

```rust
use actix_web::{web, HttpResponse, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let dashboard = web::Data::new(dashboard);
    
    HttpServer::new(move || {
        App::new()
            .app_data(dashboard.clone())
            .route("/api/dashboard/status", 
                   web::get().to(get_fleet_status))
            .route("/api/dashboard/export", 
                   web::get().to(get_export))
            .route("/api/dashboard/alerts", 
                   web::get().to(get_alerts))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn get_fleet_status(dashboard: web::Data<Dashboard>) -> HttpResponse {
    let status = dashboard.get_fleet_status();
    HttpResponse::Ok().json(status)
}
```

### Method 4: Push-Based (Scheduled Exports)

**Best for**: Scheduled compliance scans, CI/CD integration

```bash
#!/bin/bash
# Export dashboard every hour
0 * * * * /usr/local/bin/praxis-retrofit dashboard export \
  --output /var/dashboards/compliance.json \
  --repos /etc/praxis/repos.txt
```

## Grafana Integration

### Setup

1. **Add JSON Data Source**

```yaml
{
  "apiVersion": 1,
  "providers": [
    {
      "name": "Compliance Dashboard",
      "type": "file",
      "options": {
        "path": "/var/dashboards/compliance.json"
      }
    }
  ]
}
```

2. **Create Dashboard from JSON**

Use the dashboard JSON export as a Grafana dashboard source.

3. **Configure Auto-Refresh**

Set dashboard to refresh every 5 minutes:

```json
{
  "refresh": "5m",
  "panels": [
    {
      "title": "Fleet Compliance Score",
      "targets": [
        {
          "expr": "compliance_fleet{dashboard=\"praxis-compliance\"}",
          "legendFormat": "Fleet Average Score"
        }
      ]
    }
  ]
}
```

### Example Panels

#### Panel 1: Fleet Status Gauge

```json
{
  "type": "gauge",
  "title": "Fleet Average Compliance",
  "targets": [
    {
      "expr": "fleet_average_score",
      "refId": "A"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "thresholds": {
        "mode": "absolute",
        "steps": [
          { "color": "red", "value": null },
          { "color": "yellow", "value": 80 },
          { "color": "green", "value": 90 }
        ]
      }
    }
  }
}
```

#### Panel 2: Compliance by Category

```json
{
  "type": "bar",
  "title": "Compliance by Category",
  "targets": [
    {
      "expr": "avg by (category) (compliance_category)",
      "refId": "A"
    }
  ]
}
```

#### Panel 3: At-Risk Repositories

```json
{
  "type": "table",
  "title": "Repositories Below Threshold",
  "targets": [
    {
      "expr": "compliance_repo{compliance_score < 80}",
      "format": "table",
      "refId": "A"
    }
  ]
}
```

#### Panel 4: Compliance Trends

```json
{
  "type": "timeseries",
  "title": "Compliance Score Trends",
  "targets": [
    {
      "expr": "compliance_repo",
      "legendFormat": "{{repository}}",
      "refId": "A"
    }
  ],
  "options": {
    "showLegend": true,
    "legend": {
      "displayMode": "table",
      "placement": "right"
    }
  }
}
```

### Alert Rules in Grafana

```yaml
groups:
  - name: Compliance Alerts
    interval: 5m
    rules:
      - alert: RepoComplianceBelow80
        expr: compliance_repo < 80
        for: 15m
        annotations:
          summary: "{{ $labels.repository }} compliance below 80%"
          value: "{{ $value }}%"
      
      - alert: FleetAverageDeclining
        expr: rate(fleet_average_score[1h]) < -2
        for: 1h
        annotations:
          summary: "Fleet compliance trending downward"
          value: "{{ $value }}% per hour"
```

## Datadog Integration

### Setup

1. **Install Datadog Agent**

```bash
DD_AGENT_MAJOR_VERSION=7 \
DD_API_KEY=<your-api-key> \
DD_SITE="datadoghq.com" \
bash -c "$(curl -L https://s3.amazonaws.com/dd-agent/scripts/install_agent.sh)"
```

2. **Configure Compliance Dashboard Integration**

Create `/etc/datadog-agent/conf.d/compliance_dashboard.d/conf.yaml`:

```yaml
init_config:

instances:
  - compliance_json_path: /var/dashboards/compliance.json
    tags:
      - "service:praxis"
      - "team:platform"
      
custom_metrics:
  - metric_name: "compliance.fleet.average_score"
    json_path: "fleet_status.fleet_average_score"
    type: gauge
    
  - metric_name: "compliance.fleet.passing_repos"
    json_path: "fleet_status.passing_repos"
    type: gauge
    
  - metric_name: "compliance.repo.score"
    json_path: "repositories[*].compliance_score"
    type: gauge
    tags:
      - "repository:{{ repository }}"
```

3. **Push Metrics via Datadog Agent**

```bash
# Create a Python check
cat > /etc/datadog-agent/checks.d/compliance_dashboard.py << 'EOF'
from datadog_checks.base import AgentCheck

class ComplianceDashboardCheck(AgentCheck):
    def check(self, instance):
        import json
        
        json_path = instance.get('compliance_json_path')
        with open(json_path) as f:
            data = json.load(f)
        
        # Fleet metrics
        fleet = data['fleet_status']
        self.gauge('compliance.fleet.average_score', 
                   fleet['fleet_average_score'])
        self.gauge('compliance.fleet.min_score', 
                   fleet['fleet_min_score'])
        self.gauge('compliance.fleet.max_score', 
                   fleet['fleet_max_score'])
        
        # Repo metrics
        for repo in data['repositories']:
            tags = [f"repository:{repo['name']}"]
            self.gauge('compliance.repo.score', 
                       repo['compliance_score'], 
                       tags=tags)
EOF
```

### Datadog Dashboard Example

```json
{
  "title": "Praxis Compliance Dashboard",
  "widgets": [
    {
      "definition": {
        "type": "gauge",
        "title": "Fleet Average Compliance",
        "requests": [
          {
            "q": "avg:compliance.fleet.average_score{service:praxis}"
          }
        ],
        "thresholds": {
          "critical": 80,
          "warning": 90
        }
      }
    },
    {
      "definition": {
        "type": "timeseries",
        "title": "Repository Compliance Trends",
        "requests": [
          {
            "q": "avg:compliance.repo.score{service:praxis} by {repository}"
          }
        ]
      }
    },
    {
      "definition": {
        "type": "query_table",
        "title": "Repositories at Risk",
        "requests": [
          {
            "q": "top(avg:compliance.repo.score{service:praxis}, 10, 'min')"
          }
        ]
      }
    }
  ]
}
```

### Datadog Monitors

```python
# Create a monitor for low compliance
from datadog import initialize, api

options = {
    'api_key': 'your-api-key',
    'app_key': 'your-app-key'
}

initialize(**options)

monitor = {
    "type": "metric alert",
    "query": "avg(last_5m):avg:compliance.repo.score{service:praxis} < 80",
    "name": "Repository compliance below 80%",
    "message": "Repository {{repository.name}} has low compliance score: {{value}}%",
    "tags": ["service:praxis"]
}

api.Monitor.create(**monitor)
```

## Prometheus Integration

### Setup

1. **Export via Line Protocol to InfluxDB (Prometheus Remote Storage)**

Configure scrape job in `prometheus.yml`:

```yaml
global:
  scrape_interval: 5m

scrape_configs:
  - job_name: 'praxis-compliance'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
```

2. **Create Metrics Endpoint**

Implement HTTP endpoint that returns Prometheus format:

```rust
use actix_web::{web, HttpResponse};

async fn metrics(dashboard: web::Data<Dashboard>) -> HttpResponse {
    let mut output = String::new();
    
    let fleet = dashboard.get_fleet_status();
    
    // HELP and TYPE lines
    output.push_str("# HELP compliance_fleet_average Fleet average compliance score\n");
    output.push_str("# TYPE compliance_fleet_average gauge\n");
    output.push_str(&format!(
        "compliance_fleet_average{{dashboard=\"{}\"}} {}\n",
        "praxis", fleet.fleet_average_score
    ));
    
    // Per-repo metrics
    for repo in dashboard.repo_status.values() {
        output.push_str(&format!(
            "compliance_repo{{repository=\"{}\",status=\"{}\"}} {}\n",
            repo.name,
            format!("{:?}", repo.status).to_lowercase(),
            repo.compliance_score
        ));
    }
    
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(output)
}
```

3. **Prometheus Queries**

```promql
# Fleet average compliance
avg(compliance_fleet_average)

# Repos below threshold
count(compliance_repo < 80)

# Compliance by category
avg(compliance_category) by (category)

# 24-hour compliance trend
rate(compliance_repo[24h])
```

### Prometheus Alert Rules

Create `compliance-alerts.yml`:

```yaml
groups:
  - name: compliance
    interval: 5m
    rules:
      - alert: RepositoryComplianceBelow80
        expr: compliance_repo < 80
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "Repository {{ $labels.repository }} compliance below 80%"
          value: "{{ humanizePercentage $value }}"
      
      - alert: FleetComplianceDeclining
        expr: avg(rate(compliance_fleet_average[1h])) < -0.5
        for: 30m
        labels:
          severity: critical
        annotations:
          summary: "Fleet compliance declining"
          value: "{{ $value }}% per hour"
      
      - alert: MultipleRepositoriesAtRisk
        expr: count(compliance_repo < 85) > 3
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "{{ $value }} repositories near threshold"
```

## Splunk Integration

### Setup

1. **Configure HTTP Event Collector (HEC)**

```bash
# In Splunk, create HEC token
Settings > Data Inputs > HTTP Event Collector > New Token
```

2. **Send Dashboard Data to Splunk**

```rust
async fn send_to_splunk(
    dashboard: &Dashboard,
    hec_url: &str,
    hec_token: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let fleet_status = dashboard.get_fleet_status();
    
    let payload = json!({
        "event": {
            "fleet_average_score": fleet_status.fleet_average_score,
            "fleet_min_score": fleet_status.fleet_min_score,
            "fleet_max_score": fleet_status.fleet_max_score,
            "passing_repos": fleet_status.passing_repos,
            "warning_repos": fleet_status.warning_repos,
            "failing_repos": fleet_status.failing_repos,
            "repositories": dashboard.repo_status.values().collect::<Vec<_>>(),
        },
        "sourcetype": "_json"
    });
    
    client
        .post(format!("{}/services/collector", hec_url))
        .header("Authorization", format!("Splunk {}", hec_token))
        .json(&payload)
        .send()
        .await?;
    
    Ok(())
}
```

3. **Search Query Examples**

```spl
# Fleet compliance over time
sourcetype=compliance 
| timechart avg(fleet_average_score) by status

# Repository compliance distribution
sourcetype=compliance repository=*
| stats avg(compliance_score) by repository
| sort - avg(compliance_score)

# Alert on compliance drop
sourcetype=compliance
| stats latest(compliance_score) as current, 
         previous(compliance_score) as previous by repository
| eval drop = previous - current
| where drop > 5
```

## Alert Configuration

### Threshold-Based Alerts

```rust
pub struct AlertThreshold {
    pub warning_level: f32,      // e.g., 85%
    pub critical_level: f32,     // e.g., 80%
    pub rate_of_change: f32,     // e.g., -5% per day
}

let config = DashboardConfig {
    alert_threshold: 80.0,  // Critical when below 80%
    enable_alerts: true,
    ..Default::default()
};
```

### Alert Payload

```json
{
  "alert_id": "alert-repo-alpha-2026-06-23T12:34:56Z",
  "severity": "critical",
  "repository": "repo-alpha",
  "message": "Repository below threshold: 75.0% (threshold: 80.0%)",
  "triggered_at": "2026-06-23T12:34:56Z",
  "previous_score": 82.0,
  "current_score": 75.0,
  "remediation_hint": "Add deny.toml for supply chain audit"
}
```

### Integration with Alerting Services

#### PagerDuty

```rust
async fn send_alert_to_pagerduty(
    alert: &ComplianceAlert,
    integration_key: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    
    let payload = json!({
        "routing_key": integration_key,
        "event_action": if alert.acknowledged { "acknowledge" } else { "trigger" },
        "dedup_key": alert.alert_id,
        "payload": {
            "summary": alert.message,
            "severity": format!("{:?}", alert.severity).to_lowercase(),
            "source": alert.repository,
            "custom_details": {
                "previous_score": alert.previous_score,
                "current_score": alert.current_score,
                "category": alert.category,
            }
        }
    });
    
    client
        .post("https://events.pagerduty.com/v2/enqueue")
        .json(&payload)
        .send()
        .await?;
    
    Ok(())
}
```

#### Slack

```rust
async fn send_alert_to_slack(
    alert: &ComplianceAlert,
    webhook_url: &str,
) -> Result<()> {
    let color = match alert.severity {
        AlertSeverity::Critical => "#ff0000",
        AlertSeverity::Warning => "#ffaa00",
        AlertSeverity::Info => "#00aa00",
    };
    
    let payload = json!({
        "attachments": [
            {
                "color": color,
                "title": format!("{}: {}", alert.repository, alert.message),
                "fields": [
                    { "title": "Previous Score", "value": alert.previous_score, "short": true },
                    { "title": "Current Score", "value": alert.current_score, "short": true },
                    { "title": "Category", "value": alert.category.unwrap_or_default(), "short": true },
                    { "title": "Remediation", "value": alert.remediation_hint.unwrap_or_default() }
                ]
            }
        ]
    });
    
    reqwest::Client::new()
        .post(webhook_url)
        .json(&payload)
        .send()
        .await?;
    
    Ok(())
}
```

## Custom Integrations

### Building Your Own Integration

```rust
use praxis_retrofit::compliance_dashboard::Dashboard;

pub trait DashboardExporter {
    fn export(&self, dashboard: &Dashboard) -> Result<String>;
}

// Example: Custom CSV export
pub struct CsvExporter;

impl DashboardExporter for CsvExporter {
    fn export(&self, dashboard: &Dashboard) -> Result<String> {
        let mut csv = String::from("repository,score,status,ci-cd,supply-chain,linting\n");
        
        for repo in dashboard.repo_status.values() {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                repo.name,
                repo.compliance_score,
                format!("{:?}", repo.status),
                repo.category_status.get("ci-cd").map(|c| c.score).unwrap_or(0.0),
                repo.category_status.get("supply-chain").map(|c| c.score).unwrap_or(0.0),
                repo.category_status.get("linting").map(|c| c.score).unwrap_or(0.0),
            ));
        }
        
        Ok(csv)
    }
}
```

## Troubleshooting

### Common Issues

**Issue**: Dashboard shows no data
- Check that compliance reports are being added to dashboard
- Verify timestamps are in ISO 8601 format
- Ensure `config.enable_alerts` is set correctly

**Issue**: Metrics not appearing in external system
- Validate JSON/line protocol format with schema validator
- Check authentication credentials for external service
- Verify firewall/network connectivity
- Look for rate limiting in external service logs

**Issue**: Alerts not triggering
- Verify `alert_threshold` is below current scores
- Check that `enable_alerts` is true in config
- Review alert handler integrations (Slack, PagerDuty, etc.)
- Check alert acknowledgment status

**Issue**: Trends show incorrect slope
- Ensure at least 7 data points in timeline
- Verify timestamps are in ascending order
- Check that scores are realistic (0-100 range)

### Debugging

Enable debug logging:

```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

Check dashboard state:

```rust
println!("Fleet Status: {:#?}", dashboard.get_fleet_status());
println!("Active Alerts: {:#?}", dashboard.get_alerts());
println!("Trends: {:#?}", dashboard.get_all_trends());
```

## Performance Considerations

### Scalability

- **Fleet Size**: Tested with 18+ repositories
- **History Retention**: Default 90 days with auto-cleanup
- **Refresh Interval**: Recommend 5-15 minute intervals
- **Export Size**: ~50-100KB per export (JSON), ~20KB (line protocol)

### Optimization

```rust
// Configure for large fleets
let config = DashboardConfig {
    history_retention_days: 30,  // Shorter retention
    enable_alerts: true,
    alert_threshold: 85.0,
    ..Default::default()
};

// Clean up old history monthly
dashboard.cleanup_old_history();

// Export periodically, not on every update
dashboard.snapshot();  // Record for history
let json = dashboard.export_json()?;  // Export only when needed
```

## References

- [Dashboard Export Schema](../examples/dashboard-schema.json)
- [Compliance Dashboard Module](../src/compliance_dashboard.rs)
- [Grafana Alerting Docs](https://grafana.com/docs/grafana/latest/alerting/)
- [Datadog API Reference](https://docs.datadoghq.com/api/latest/)
- [Prometheus Querying](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Splunk Search Reference](https://docs.splunk.com/Documentation/Splunk/latest/SearchReference)
