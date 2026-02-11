-- Azure SQL definitions lookup query
-- Input parameter (ODBC positional):
--   1) table LIKE pattern (example: 'dim_project%')
-- Expected columns in result:
--   - column
--   - definition
-- Notes:
--   - `?` is bound in Rust via ODBC parameter binding.
--   - Keep selected column order aligned with downstream markdown formatting.

SELECT
    [column]
    , [definition]
FROM
    [database].[dbo].[definitions]
WHERE
    [table] LIKE ?
