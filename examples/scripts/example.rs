use std::collections::BTreeMap;

#[derive(Debug)]
struct Order {
    order_id: &'static str,
    customer_id: &'static str,
    order_date: &'static str,
    amount: f64,
    status: &'static str,
}

fn load_bronze_orders() -> Vec<Order> {
    vec![
        Order { order_id: "1001", customer_id: "C001", order_date: "2026-02-13", amount: 125.50, status: "completed" },
        Order { order_id: "1002", customer_id: "C002", order_date: "2026-02-13", amount: 20.00, status: "cancelled" },
        Order { order_id: "1003", customer_id: "C001", order_date: "2026-02-13", amount: 88.00, status: "completed" },
        Order { order_id: "1004", customer_id: "C003", order_date: "2026-02-14", amount: 42.25, status: "completed" },
    ]
}

fn transform_to_gold(orders: &[Order]) -> BTreeMap<(String, String), (f64, i32)> {
    let mut aggregate: BTreeMap<(String, String), (f64, i32)> = BTreeMap::new();

    for order in orders {
        if order.status != "completed" {
            continue;
        }

        let key = (order.order_date.to_string(), order.customer_id.to_string());
        let entry = aggregate.entry(key).or_insert((0.0, 0));
        entry.0 += order.amount;
        entry.1 += 1;
    }

    aggregate
}

fn main() {
    let bronze_orders = load_bronze_orders();
    let gold_rows = transform_to_gold(&bronze_orders);

    println!("gold.fct_daily_customer_sales");
    println!("order_date,customer_id,daily_revenue,order_count");

    for ((order_date, customer_id), (daily_revenue, order_count)) in gold_rows {
        println!("{},{},{:.2},{}", order_date, customer_id, daily_revenue, order_count);
    }

    let order_ids: Vec<&str> = bronze_orders.iter().map(|o| o.order_id).collect();
    println!("processed_order_ids={:?}", order_ids);
}
