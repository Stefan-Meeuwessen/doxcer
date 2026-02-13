param(
    [string]$LoadDate = "2026-02-13",
    [switch]$ForceFullLoad
)

$bronzeOrders = @(
    [PSCustomObject]@{ order_id = "1001"; customer_id = "C001"; order_date = "2026-02-13"; order_amount = 125.50; status = "completed" }
    [PSCustomObject]@{ order_id = "1002"; customer_id = "C002"; order_date = "2026-02-13"; order_amount = 20.00; status = "cancelled" }
    [PSCustomObject]@{ order_id = "1003"; customer_id = "C001"; order_date = "2026-02-13"; order_amount = 88.00; status = "completed" }
    [PSCustomObject]@{ order_id = "1004"; customer_id = "C003"; order_date = "2026-02-14"; order_amount = 42.25; status = "completed" }
)

if (-not $ForceFullLoad) {
    $bronzeOrders = $bronzeOrders | Where-Object { $_.order_date -eq $LoadDate }
}

$goldRows = $bronzeOrders |
    Where-Object { $_.status -eq "completed" } |
    Group-Object -Property order_date, customer_id |
    ForEach-Object {
        $parts = $_.Name -split ',\s*'
        [PSCustomObject]@{
            order_date = $parts[0]
            customer_id = $parts[1]
            daily_revenue = [Math]::Round(($_.Group | Measure-Object -Property order_amount -Sum).Sum, 2)
            order_count = $_.Count
        }
    } |
    Sort-Object order_date, customer_id

Write-Output "gold.fct_daily_customer_sales"
$goldRows | Format-Table -AutoSize
