-- Example ETL SQL script: bronze -> gold aggregation
-- This script is intentionally self-contained for documentation tests.

WITH bronze_orders AS (
    SELECT '1001' AS order_id, 'C001' AS customer_id, CAST('2026-02-13' AS DATE) AS order_date, 125.50 AS order_amount, 'completed' AS status
    UNION ALL
    SELECT '1002', 'C002', CAST('2026-02-13' AS DATE), 20.00, 'cancelled'
    UNION ALL
    SELECT '1003', 'C001', CAST('2026-02-13' AS DATE), 88.00, 'completed'
    UNION ALL
    SELECT '1004', 'C003', CAST('2026-02-14' AS DATE), 42.25, 'completed'
),
gold_fct_daily_customer_sales AS (
    SELECT
        order_date,
        customer_id,
        ROUND(SUM(order_amount), 2) AS daily_revenue,
        COUNT(*) AS order_count
    FROM bronze_orders
    WHERE status = 'completed'
    GROUP BY order_date, customer_id
)
SELECT
    order_date,
    customer_id,
    daily_revenue,
    order_count
FROM gold_fct_daily_customer_sales
ORDER BY order_date, customer_id;
