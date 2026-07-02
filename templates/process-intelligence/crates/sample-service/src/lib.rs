pub mod ocel_event;

use chicago_tdd_tools::core::governance::{
    close_channel, emit_diagnostic, register_sink, set_run_id, Diagnostic, DiagnosticCategory,
    DiagnosticCode, Severity,
};
use chicago_tdd_tools::observability::ocel::OcelCollector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static EVENT_COUNTER: AtomicU64 = AtomicU64::new(1000);

fn next_timestamp() -> u64 {
    EVENT_COUNTER.fetch_add(1000, Ordering::SeqCst)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub order_id: String,
    pub customer_id: String,
    pub items: Vec<OrderItem>,
    pub status: OrderStatus,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderItem {
    pub item_id: String,
    pub price: f64,
    pub quantity: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Created,
    Validated,
    Paid,
    Dispatched,
    Cancelled,
}

pub fn create_order(customer_id: String, items: Vec<OrderItem>) -> Order {
    let mut total_amount = 0.0;
    for item in &items {
        total_amount += item.price * (item.quantity as f64);
    }

    Order {
        order_id: format!("ord-{}", uuid::Uuid::new_v4()),
        customer_id,
        items,
        status: OrderStatus::Created,
        total_amount,
    }
}

pub fn validate_order(order: &mut Order) -> Result<(), &'static str> {
    if order.status != OrderStatus::Created {
        return Err("Order is not in Created status");
    }
    if order.items.is_empty() {
        return Err("Order has no items");
    }
    for item in &order.items {
        if item.price <= 0.0 {
            return Err("Item price must be positive");
        }
        if item.quantity == 0 {
            return Err("Item quantity must be positive");
        }
    }
    order.status = OrderStatus::Validated;
    Ok(())
}

pub fn pay_order(order: &mut Order) -> Result<(), &'static str> {
    if order.status != OrderStatus::Validated {
        return Err("Order is not in Validated status");
    }
    order.status = OrderStatus::Paid;
    Ok(())
}

pub fn dispatch_order(order: &mut Order) -> Result<(), &'static str> {
    if order.status != OrderStatus::Paid {
        return Err("Order is not in Paid status");
    }
    order.status = OrderStatus::Dispatched;
    Ok(())
}

pub fn process_order_workflow(
    case_id: &str,
    customer_id: String,
    items: Vec<OrderItem>,
    ocel_path: Option<PathBuf>,
) -> Result<Order, &'static str> {
    if let Some(ref path) = ocel_path {
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        let collector = OcelCollector::new(Some(path.clone()));
        register_sink(Box::new(collector));
    }

    set_run_id(case_id.to_string());

    let log_event = |activity: &str, order_id: &str, msg: &str| {
        let mut context = HashMap::new();
        let _ = context.insert("order_id", serde_json::Value::String(order_id.to_string()));
        let _ = context.insert("activity", serde_json::Value::String(activity.to_string()));
        let _ = context.insert("message", serde_json::Value::String(msg.to_string()));

        let diag = Diagnostic {
            code: DiagnosticCode::new(
                "order_processing".to_string(),
                DiagnosticCategory::Conformance,
                100,
            ),
            category: DiagnosticCategory::Conformance,
            severity: Severity::Info,
            location: None,
            message: format!("{activity}: {msg}"),
            context,
            run_id: case_id.to_string(),
            agent_id: None,
            source_module: "sample_service",
            elapsed_ns: next_timestamp(),
        };
        emit_diagnostic(&diag);
    };

    let mut order = create_order(customer_id, items);
    log_event("OrderCreation", &order.order_id, "Order created successfully");

    validate_order(&mut order)?;
    log_event("OrderValidation", &order.order_id, "Order validated successfully");

    pay_order(&mut order)?;
    log_event("OrderPayment", &order.order_id, "Order paid successfully");

    dispatch_order(&mut order)?;
    log_event("OrderDispatch", &order.order_id, "Order dispatched successfully");

    if ocel_path.is_some() {
        let _ = close_channel();
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chicago_tdd_tools::prelude::*;
    use chicago_tdd_tools::snapshot::SnapshotAssert;
    use proptest::prelude::*;

    #[derive(Debug)]
    struct TestError(String);

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    impl From<std::io::Error> for TestError {
        fn from(err: std::io::Error) -> Self {
            TestError(err.to_string())
        }
    }

    impl From<serde_json::Error> for TestError {
        fn from(err: serde_json::Error) -> Self {
            TestError(err.to_string())
        }
    }

    impl From<&'static str> for TestError {
        fn from(err: &'static str) -> Self {
            TestError(err.to_string())
        }
    }

    chicago_tdd_tools::test!(test_create_order_success, {
        let items = vec![
            OrderItem {
                item_id: "item1".to_string(),
                price: 10.0,
                quantity: 2,
            },
            OrderItem {
                item_id: "item2".to_string(),
                price: 15.0,
                quantity: 1,
            },
        ];
        let order = create_order("cust1".to_string(), items);
        assert_eq!(order.customer_id, "cust1");
        assert_eq!(order.total_amount, 35.0);
        assert_eq!(order.status, OrderStatus::Created);
        Ok::<(), TestError>(())
    });

    chicago_tdd_tools::test!(test_validate_order_empty_items, {
        let mut order = create_order("cust1".to_string(), vec![]);
        let res = validate_order(&mut order);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Order has no items");
        Ok::<(), TestError>(())
    });

    chicago_tdd_tools::test!(test_order_workflow_with_telemetry, {
        let items = vec![OrderItem {
            item_id: "item1".to_string(),
            price: 100.0,
            quantity: 1,
        }];
        let dir = tempfile::tempdir()?;
        let log_path = dir.path().join("order_flow.ocel.json");

        let order = process_order_workflow(
            "case-999",
            "customer_xyz".to_string(),
            items,
            Some(log_path.clone()),
        )?;
        assert_eq!(order.status, OrderStatus::Dispatched);

        assert!(log_path.exists());
        let log_content = std::fs::read_to_string(&log_path)?;
        assert!(log_content.contains("order_id"));
        assert!(log_content.contains("case-999"));
        Ok::<(), TestError>(())
    });

    chicago_tdd_tools::test!(test_snapshot_simple_order, {
        let items = vec![OrderItem {
            item_id: "item_snap".to_string(),
            price: 29.99,
            quantity: 1,
        }];
        let order = create_order("cust_snap".to_string(), items);
        let order_json = serde_json::to_value(&order)?;
        let mut redactions = std::collections::HashMap::new();
        redactions.insert(".order_id".to_string(), "[ORDER_ID]".to_string());
        SnapshotAssert::with_settings(
            |settings| {
                let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                path.push("src/snapshots");
                settings.set_snapshot_path(path);
            },
            || {
                SnapshotAssert::assert_with_redaction(&order_json, "sample_order_snapshot", &redactions);
            },
        );
        Ok::<(), TestError>(())
    });

    chicago_tdd_tools::test!(test_property_generator_invariants, {
        let mut generator = PropertyTestGenerator::<20, 5>::new().with_seed(1234);
        let data = generator.generate_test_data();
        assert!(!data.is_empty());
        for (k, v) in data {
            assert!(k.starts_with("key_"));
            assert!(v.starts_with("value_"));
        }
        Ok::<(), TestError>(())
    });

    struct TestLcgRng {
        state: u64,
    }

    impl TestLcgRng {
        fn new(seed: u64) -> Self {
            Self { state: seed }
        }

        fn next(&mut self) -> u64 {
            self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            self.state
        }
    }

    chicago_tdd_tools::test!(test_json_fuzzing_robustness, {
        let mut rng = TestLcgRng::new(54321);
        for _ in 0..50 {
            let mut random_str = String::new();
            random_str.push_str("{\"order_id\": \"");
            for _ in 0..5 {
                let c = ((rng.next() % 26) as u8 + b'A') as char;
                random_str.push(c);
            }
            random_str.push_str("\", \"total_amount\": ");
            if rng.next().is_multiple_of(2) {
                random_str.push_str("invalid_number");
            } else {
                random_str.push_str("42.0");
            }
            random_str.push('}');

            let _parsed: Result<Order, _> = serde_json::from_str(&random_str);
        }
        Ok::<(), TestError>(())
    });

    fn arb_order_item() -> impl Strategy<Value = OrderItem> {
        (
            any::<String>(),
            -100.0..1000.0f64,
            0..100u32,
        ).prop_map(|(item_id, price, quantity)| OrderItem {
            item_id,
            price,
            quantity,
        })
    }

    proptest! {
        #[test]
        fn test_proptest_order_total_is_correct(
            price1 in 0.1..100.0f64,
            qty1 in 1..10u32,
            price2 in 0.1..100.0f64,
            qty2 in 1..10u32,
        ) {
            let items = vec![
                OrderItem { item_id: "item1".to_string(), price: price1, quantity: qty1 },
                OrderItem { item_id: "item2".to_string(), price: price2, quantity: qty2 },
            ];
            let order = create_order("customer_prop".to_string(), items);
            let expected_total = price1 * (qty1 as f64) + price2 * (qty2 as f64);
            prop_assert!((order.total_amount - expected_total).abs() < 1e-5);
        }

        #[test]
        fn test_proptest_order_creation_invariants(
            customer_id in any::<String>(),
            items in prop::collection::vec(arb_order_item(), 0..10),
        ) {
            let order = create_order(customer_id.clone(), items.clone());
            prop_assert!(order.order_id.starts_with("ord-"));
            prop_assert_eq!(&order.customer_id, &customer_id);
            prop_assert_eq!(order.status, OrderStatus::Created);

            let mut expected_total = 0.0;
            for item in &items {
                expected_total += item.price * (item.quantity as f64);
            }
            prop_assert!((order.total_amount - expected_total).abs() < 1e-5);
        }

        #[test]
        fn test_proptest_order_validation_invariants(
            customer_id in any::<String>(),
            items in prop::collection::vec(arb_order_item(), 0..10),
        ) {
            let mut order = create_order(customer_id, items.clone());
            let res = validate_order(&mut order);

            let has_invalid_item = items.iter().any(|item| item.price <= 0.0 || item.quantity == 0);
            let should_fail = items.is_empty() || has_invalid_item;

            if should_fail {
                prop_assert!(res.is_err());
                prop_assert_eq!(order.status, OrderStatus::Created);
            } else {
                prop_assert!(res.is_ok());
                prop_assert_eq!(order.status, OrderStatus::Validated);
            }
        }

        #[test]
        fn test_proptest_state_transition_invariants(
            customer_id in any::<String>(),
            items in prop::collection::vec(
                (any::<String>(), 0.01..1000.0f64, 1..100u32)
                    .prop_map(|(id, p, q)| OrderItem { item_id: id, price: p, quantity: q }),
                1..10
            ),
        ) {
            let mut order = create_order(customer_id, items);

            // Initially Created.
            prop_assert_eq!(order.status, OrderStatus::Created);

            // Cannot pay or dispatch yet
            prop_assert!(pay_order(&mut order).is_err());
            prop_assert!(dispatch_order(&mut order).is_err());
            prop_assert_eq!(order.status, OrderStatus::Created);

            // Validate the order
            prop_assert!(validate_order(&mut order).is_ok());
            prop_assert_eq!(order.status, OrderStatus::Validated);

            // Cannot validate again, cannot dispatch yet
            prop_assert!(validate_order(&mut order).is_err());
            prop_assert!(dispatch_order(&mut order).is_err());
            prop_assert_eq!(order.status, OrderStatus::Validated);

            // Pay the order
            prop_assert!(pay_order(&mut order).is_ok());
            prop_assert_eq!(order.status, OrderStatus::Paid);

            // Cannot validate or pay again
            prop_assert!(validate_order(&mut order).is_err());
            prop_assert!(pay_order(&mut order).is_err());
            prop_assert_eq!(order.status, OrderStatus::Paid);

            // Dispatch the order
            prop_assert!(dispatch_order(&mut order).is_ok());
            prop_assert_eq!(order.status, OrderStatus::Dispatched);

            // Cannot validate, pay, or dispatch again
            prop_assert!(validate_order(&mut order).is_err());
            prop_assert!(pay_order(&mut order).is_err());
            prop_assert!(dispatch_order(&mut order).is_err());
            prop_assert_eq!(order.status, OrderStatus::Dispatched);
        }
    }
}
