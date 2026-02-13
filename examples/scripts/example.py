from collections import defaultdict
from datetime import datetime
from typing import Dict, List, Tuple


Order = Dict[str, object]


def load_bronze_orders() -> List[Order]:
    return [
        {"order_id": "1001", "customer_id": "C001", "order_ts": "2026-02-13T09:10:00", "amount": 125.50, "status": "completed"},
        {"order_id": "1002", "customer_id": "C002", "order_ts": "2026-02-13T10:30:00", "amount": 20.00, "status": "cancelled"},
        {"order_id": "1003", "customer_id": "C001", "order_ts": "2026-02-13T11:45:00", "amount": 88.00, "status": "completed"},
        {"order_id": "1004", "customer_id": "C003", "order_ts": "2026-02-14T08:00:00", "amount": 42.25, "status": "completed"},
    ]


def transform_to_gold(orders: List[Order]) -> List[Dict[str, object]]:
    aggregates: Dict[Tuple[str, str], Dict[str, object]] = defaultdict(lambda: {"daily_revenue": 0.0, "order_count": 0})

    for order in orders:
        if order["status"] != "completed":
            continue

        order_date = datetime.fromisoformat(str(order["order_ts"])).date().isoformat()
        key = (order_date, str(order["customer_id"]))
        aggregates[key]["daily_revenue"] += float(order["amount"])
        aggregates[key]["order_count"] += 1

    result = []
    for (order_date, customer_id), metrics in sorted(aggregates.items()):
        result.append(
            {
                "order_date": order_date,
                "customer_id": customer_id,
                "daily_revenue": round(metrics["daily_revenue"], 2),
                "order_count": metrics["order_count"],
            }
        )

    return result


def main() -> None:
    bronze_orders = load_bronze_orders()
    gold_rows = transform_to_gold(bronze_orders)

    print("gold.fct_daily_customer_sales")
    for row in gold_rows:
        print(row)


if __name__ == "__main__":
    main()
