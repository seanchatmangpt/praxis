# Praxis Compliance Dashboard - Complete Index

Welcome to the Compliance Dashboard documentation. This index helps you navigate all resources for implementing and using fleet-wide compliance monitoring.

## Quick Navigation

### For Getting Started (5-15 minutes)

1. **[DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md)** - Start here!
   - 5-minute setup
   - Basic API usage
   - Common operations
   - Configuration examples

2. **Examples** - Run these to see it in action:
   - `cargo run --example dashboard_usage` - Full feature demonstration
   - `cargo run --example dashboard-configs` - Configuration templates

### For In-Depth Understanding (30+ minutes)

3. **[DASHBOARD_README.md](DASHBOARD_README.md)** - Complete reference
   - Architecture and design
   - Data models
   - API reference
   - Use cases and examples
   - Performance characteristics

4. **[DASHBOARD_IMPLEMENTATION_SUMMARY.md](DASHBOARD_IMPLEMENTATION_SUMMARY.md)** - What was built
   - Deliverables overview
   - Features implemented
   - Architecture details
   - Usage patterns
   - Getting started

### For External System Integration (45+ minutes)

5. **[DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md)** - Integration with:
   - Grafana (dashboards, alerts)
   - Datadog (agents, monitors)
   - Prometheus (metrics, queries, alerts)
   - Splunk (ingestion, searches)
   - Custom systems (webhooks, REST API)

### Reference Materials

6. **examples/dashboard-schema.json** - JSON Schema
   - Complete data model definition
   - Field descriptions and types
   - Validation rules
   - Integration specification

7. **Source Code: src/compliance_dashboard.rs**
   - Production-quality Rust implementation
   - Comprehensive documentation
   - Unit tests included

## By Use Case

### "I want to monitor my fleet RIGHT NOW"

1. Read: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md) (5 min)
2. Run: `cargo run --example dashboard_usage` (2 min)
3. Integrate: Choose your platform in [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md) (30 min)

### "I need to understand the system architecture"

1. Read: [DASHBOARD_README.md](DASHBOARD_README.md) - Architecture section (10 min)
2. Read: [DASHBOARD_IMPLEMENTATION_SUMMARY.md](DASHBOARD_IMPLEMENTATION_SUMMARY.md) (15 min)
3. Review: Source code `src/compliance_dashboard.rs` (20 min)

### "I want to integrate with Grafana"

1. Quick overview: [DASHBOARD_README.md](DASHBOARD_README.md) (5 min)
2. Integration guide: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#grafana-integration) (20 min)
3. Example configuration: dashboard-schema.json (5 min)

### "I want to integrate with Datadog"

1. Overview: [DASHBOARD_README.md](DASHBOARD_README.md) (5 min)
2. Integration guide: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#datadog-integration) (20 min)
3. Run example: `cargo run --example dashboard_usage` (2 min)

### "I want to integrate with Prometheus"

1. Overview: [DASHBOARD_README.md](DASHBOARD_README.md) (5 min)
2. Integration guide: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#prometheus-integration) (20 min)
3. Example queries: Same document (10 min)

### "I need custom configuration for my environment"

1. Configuration options: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#configuration) (5 min)
2. Examples: `cargo run --example dashboard-configs` (2 min)
3. Custom config: [DASHBOARD_README.md](DASHBOARD_README.md#configuration) (10 min)

### "I want to understand the data model"

1. Models overview: [DASHBOARD_README.md](DASHBOARD_README.md#data-models) (10 min)
2. JSON schema: examples/dashboard-schema.json (15 min)
3. Source documentation: src/compliance_dashboard.rs (20 min)

### "I need to troubleshoot issues"

1. Common issues: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#troubleshooting) (5 min)
2. Advanced troubleshooting: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#troubleshooting) (10 min)
3. Source code debugging: src/compliance_dashboard.rs (vary)

## Document Roadmap

### Foundational Documents (Required Reading)

```
Start Here
    ↓
DASHBOARD_QUICKSTART.md (5-min overview + examples)
    ↓
DASHBOARD_README.md (Complete reference guide)
    ↓
Your use case specific integration guide
```

### Supplementary Documents

```
DASHBOARD_IMPLEMENTATION_SUMMARY.md
    ├─ Explains what was built
    ├─ Architecture details
    └─ Usage patterns

examples/dashboard-schema.json
    ├─ Data model specification
    ├─ External system integration
    └─ Validation rules

Source Code: src/compliance_dashboard.rs
    ├─ Implementation details
    ├─ API documentation
    └─ Test suite
```

## Feature Reference

### Real-Time Monitoring

**Quick Start**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#step-3-get-fleet-status)

**Complete Guide**: [DASHBOARD_README.md](DASHBOARD_README.md#real-time-fleet-status)

**Implementation**: `src/compliance_dashboard.rs` - `get_fleet_status()`

### Trend Tracking

**Quick Start**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#step-8-trend-analysis)

**Complete Guide**: [DASHBOARD_README.md](DASHBOARD_README.md#trend-analysis)

**Implementation**: `src/compliance_dashboard.rs` - `update_trend()`

### Alert System

**Quick Start**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#step-4-check-alerts)

**Configuration**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#alert-handling)

**Complete Guide**: [DASHBOARD_README.md](DASHBOARD_README.md#alert-system)

**Integration**: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#alert-configuration)

### JSON Export

**Quick Start**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#step-5-export-data)

**Schema**: examples/dashboard-schema.json

**Implementation**: `src/compliance_dashboard.rs` - `export_json()`

### Line Protocol Export

**Quick Start**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#export-formats)

**Implementation**: `src/compliance_dashboard.rs` - `export_line_protocol()`

**Usage with Prometheus**: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#prometheus-integration)

### Historical Tracking

**Quick Start**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#step-9-record-history)

**Complete Guide**: [DASHBOARD_README.md](DASHBOARD_README.md#metrics-reference)

**Implementation**: `src/compliance_dashboard.rs` - `snapshot()`

## Integration Platform Guides

### Grafana

**Location**: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#grafana-integration)

**Time**: 30 minutes

**Topics**:
- JSON data source setup
- Dashboard panel examples
- Alert rule configuration
- Best practices

### Datadog

**Location**: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#datadog-integration)

**Time**: 30 minutes

**Topics**:
- Agent configuration
- Custom Python checks
- Dashboard creation
- Monitor setup

### Prometheus

**Location**: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#prometheus-integration)

**Time**: 30 minutes

**Topics**:
- Metrics endpoint implementation
- PromQL queries
- Alert rules
- Remote storage

### Splunk

**Location**: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#splunk-integration)

**Time**: 30 minutes

**Topics**:
- HEC configuration
- Data ingestion
- Search queries
- Dashboard examples

## API Quick Reference

### Core Types

| Type | Purpose | Reference |
|------|---------|-----------|
| `Dashboard` | Main container | [README](DASHBOARD_README.md#api-reference) |
| `DashboardConfig` | Configuration | [QUICKSTART](DASHBOARD_QUICKSTART.md#configuration) |
| `FleetStatus` | Aggregated metrics | [README](DASHBOARD_README.md#fleetsatus) |
| `RepositoryStatus` | Per-repo status | [README](DASHBOARD_README.md#repositorystatus) |
| `ComplianceAlert` | Alert notification | [README](DASHBOARD_README.md#compliancealert) |
| `ComplianceTrend` | Historical trend | [README](DASHBOARD_README.md#compliancetrend) |

### Key Methods

| Method | Purpose | Reference |
|--------|---------|-----------|
| `new()` | Create dashboard | [QUICKSTART](DASHBOARD_QUICKSTART.md#1-create-dashboard) |
| `add_report()` | Add compliance report | [QUICKSTART](DASHBOARD_QUICKSTART.md#2-add-compliance-reports) |
| `get_fleet_status()` | Query fleet metrics | [QUICKSTART](DASHBOARD_QUICKSTART.md#query-fleet-status) |
| `get_alerts()` | Get active alerts | [QUICKSTART](DASHBOARD_QUICKSTART.md#handle-alerts) |
| `export_json()` | Export as JSON | [QUICKSTART](DASHBOARD_QUICKSTART.md#json-export) |
| `export_line_protocol()` | Export for time-series | [QUICKSTART](DASHBOARD_QUICKSTART.md#line-protocol-export) |

## Configuration Templates

### Strict (Security-Critical)

```rust
config.alert_threshold = 90.0;
config.category_weights.insert("supply-chain", 1.5);
```

See: `examples/dashboard-configs.rs` - `strict_config()`

### Development

```rust
config.alert_threshold = 70.0;
config.history_retention_days = 14;
```

See: `examples/dashboard-configs.rs` - `dev_config()`

### Production

```rust
config.alert_threshold = 85.0;
config.history_retention_days = 90;
```

See: `examples/dashboard-configs.rs` - `production_config()`

### Security-Focused

```rust
config.category_weights.insert("supply-chain", 2.0);
config.history_retention_days = 365;
```

See: `examples/dashboard-configs.rs` - `security_focused_config()`

## Troubleshooting Guide

### No data in dashboard?

**Quick Fix**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#troubleshooting)

**Detailed Help**: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#troubleshooting)

### Alerts not triggering?

**Quick Fix**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#alerts-not-triggering)

**Full Guide**: [DASHBOARD_README.md](DASHBOARD_README.md#troubleshooting)

### Integration not working?

**Platform-Specific**: [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md#troubleshooting)

**General**: [DASHBOARD_README.md](DASHBOARD_README.md#troubleshooting)

## Performance & Optimization

**Reference**: [DASHBOARD_README.md](DASHBOARD_README.md#performance)

**Tips**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#performance-tips)

**Patterns**: [DASHBOARD_IMPLEMENTATION_SUMMARY.md](DASHBOARD_IMPLEMENTATION_SUMMARY.md#usage-patterns)

## Testing

**Unit Tests**: `src/compliance_dashboard.rs` - test module

**Examples**: 
- `cargo run --example dashboard_usage`
- `cargo run --example dashboard-configs`

**Running Tests**:
```bash
cargo test compliance_dashboard::
```

## File Manifest

### Documentation (4 files, ~3000 lines)

- `docs/DASHBOARD_INDEX.md` (this file) - Navigation guide
- `docs/DASHBOARD_README.md` - Complete reference
- `docs/DASHBOARD_QUICKSTART.md` - Quick start
- `docs/DASHBOARD_INTEGRATION.md` - Integration guides

### Implementation (1 file, ~750 lines)

- `src/compliance_dashboard.rs` - Production Rust module
  - Includes comprehensive unit tests
  - Full API documentation
  - 100+ inline comments

### Examples (3 files, ~450 lines)

- `examples/dashboard_usage.rs` - Full feature demonstration
- `examples/dashboard-configs.rs` - Configuration examples
- `examples/dashboard-schema.json` - JSON Schema definition

## Getting Help

### Issue: Can't find what I need

**Start here**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md)

### Issue: Want to integrate with specific platform

**Find it**: Search by platform name in [DASHBOARD_INTEGRATION.md](DASHBOARD_INTEGRATION.md)

### Issue: API question

**Reference**: [DASHBOARD_README.md](DASHBOARD_README.md#api-reference)

**Example**: Check `examples/dashboard_usage.rs`

### Issue: Configuration question

**Guide**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md#configuration)

**Examples**: Run `cargo run --example dashboard-configs`

## Document Structure Diagram

```
DASHBOARD_INDEX.md (You are here)
    │
    ├─→ DASHBOARD_QUICKSTART.md
    │   ├─ 5-min setup
    │   ├─ Configuration
    │   ├─ Common operations
    │   └─ Troubleshooting
    │
    ├─→ DASHBOARD_README.md
    │   ├─ Features
    │   ├─ Architecture
    │   ├─ Data models
    │   ├─ API reference
    │   ├─ Use cases
    │   └─ Performance
    │
    ├─→ DASHBOARD_INTEGRATION.md
    │   ├─ Grafana
    │   ├─ Datadog
    │   ├─ Prometheus
    │   ├─ Splunk
    │   ├─ Alert integration
    │   └─ Troubleshooting
    │
    ├─→ DASHBOARD_IMPLEMENTATION_SUMMARY.md
    │   ├─ What was built
    │   ├─ Architecture
    │   ├─ Usage patterns
    │   └─ Getting started
    │
    ├─→ examples/dashboard_usage.rs
    │   └─ Runnable demonstration
    │
    ├─→ examples/dashboard-configs.rs
    │   └─ Configuration templates
    │
    ├─→ examples/dashboard-schema.json
    │   └─ Data model specification
    │
    └─→ src/compliance_dashboard.rs
        └─ Production implementation
```

## Next Steps

1. **Start**: Open [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md)
2. **Run**: Execute `cargo run --example dashboard_usage`
3. **Choose**: Pick your integration platform
4. **Integrate**: Follow the specific integration guide
5. **Monitor**: Start using your dashboard!

---

**Version**: 1.0  
**Last Updated**: 2026-06-23  
**Status**: Production Ready

For API documentation, run: `cargo doc --open`
