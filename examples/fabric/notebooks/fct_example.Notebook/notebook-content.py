# Fabric notebook source

# METADATA ********************
# META {
# META   "kernel_info": {
# META     "name": "synapse_pyspark"
# META   },
# META   "dependencies": {
# META     "lakehouse": {
# META       "default_lakehouse": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
# META       "default_lakehouse_name": "example_lakehouse",
# META       "default_lakehouse_workspace_id": "11111111-1111-1111-1111-111111111111"
# META     },
# META     "environment": {
# META       "environmentId": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
# META       "workspaceId": "11111111-1111-1111-1111-111111111111"
# META     }
# META   }
# META }

# MARKDOWN ********************
# # fct_example
# Fabric notebook fixture for doxcer parsing tests.

# CELL ********************
from pyspark.sql import functions as F

bronze_orders_df = spark.createDataFrame(
    [
        ("1001", "C001", "2026-01-01T09:10:00", 125.50, "completed"),
        ("1002", "C002", "2026-01-01T10:30:00", 20.00, "cancelled"),
        ("1003", "C001", "2026-01-02T08:15:00", 88.00, "completed")
    ],
    ["order_id", "customer_id", "order_ts", "order_amount", "status"]
)

# CELL ********************
silver_orders_df = (
    bronze_orders_df
    .filter(F.col("status") == "completed")
    .withColumn("order_date", F.to_date("order_ts"))
)

# CELL ********************
gold_fct_daily_customer_sales_df = (
    silver_orders_df
    .groupBy("order_date", "customer_id")
    .agg(
        F.sum("order_amount").alias("daily_revenue"),
        F.count("*").alias("order_count")
    )
)

gold_fct_daily_customer_sales_df.show()

# CELL ********************
(
    gold_fct_daily_customer_sales_df
    .write
    .mode("overwrite")
    .format("delta")
    .saveAsTable("gold.fct_daily_customer_sales")
)

# METADATA ********************
# META {
# META   "language": "python",
# META   "language_group": "synapse_pyspark"
# META }
