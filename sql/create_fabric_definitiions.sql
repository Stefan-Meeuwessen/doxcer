-- Microsoft Fabric Warehouse / SQL analytics endpoint compatible DDL
-- Notes:
-- - IDENTITY in Fabric Warehouse supports BIGINT only.
-- - DEFAULT constraints are not supported.
-- - Triggers are not supported.
-- - PRIMARY KEY constraints must be added with ALTER TABLE ... NOT ENFORCED.
--
-- Because DEFAULT constraints and triggers are unavailable, "automatic" timestamps
-- are implemented through stored procedures below.

CREATE TABLE dbo.definitions
(
    [id] BIGINT IDENTITY,

    [lakehouse] VARCHAR(256) NULL,
    [database] VARCHAR(256) NULL,
    [table] VARCHAR(256) NULL,
    [column] VARCHAR(256) NULL,

    [dashboard] VARCHAR(512) NULL,
    [definition] VARCHAR(8000) NULL,
    [definition_owner] VARCHAR(256) NULL,

    [creation_date] DATETIME2(3) NOT NULL,
    [modified_date] DATETIME2(3) NOT NULL
);

-- Optional key metadata for Fabric Warehouse:
-- ALTER TABLE dbo.definitions
-- ADD CONSTRAINT PK_definitions PRIMARY KEY NONCLUSTERED ([id]) NOT ENFORCED;


-- Insert procedure:
-- Automatically sets both creation_date and modified_date.
CREATE OR ALTER PROCEDURE dbo.insert_definition
    @lakehouse VARCHAR(256) = NULL,
    @database VARCHAR(256) = NULL,
    @table VARCHAR(256) = NULL,
    @column VARCHAR(256) = NULL,
    @dashboard VARCHAR(512) = NULL,
    @definition VARCHAR(8000) = NULL,
    @definition_owner VARCHAR(256) = NULL
AS
BEGIN
    DECLARE @now DATETIME2(3) = SYSUTCDATETIME();

    INSERT INTO dbo.definitions
    (
        [lakehouse],
        [database],
        [table],
        [column],
        [dashboard],
        [definition],
        [definition_owner],
        [creation_date],
        [modified_date]
    )
    VALUES
    (
        @lakehouse,
        @database,
        @table,
        @column,
        @dashboard,
        @definition,
        @definition_owner,
        @now,
        @now
    );
END;


-- Update procedure:
-- Always refreshes modified_date for the updated record.
CREATE OR ALTER PROCEDURE dbo.update_definition
    @id BIGINT,
    @lakehouse VARCHAR(256) = NULL,
    @database VARCHAR(256) = NULL,
    @table VARCHAR(256) = NULL,
    @column VARCHAR(256) = NULL,
    @dashboard VARCHAR(512) = NULL,
    @definition VARCHAR(8000) = NULL,
    @definition_owner VARCHAR(256) = NULL
AS
BEGIN
    UPDATE dbo.definitions
    SET
        [lakehouse] = COALESCE(@lakehouse, [lakehouse]),
        [database] = COALESCE(@database, [database]),
        [table] = COALESCE(@table, [table]),
        [column] = COALESCE(@column, [column]),
        [dashboard] = COALESCE(@dashboard, [dashboard]),
        [definition] = COALESCE(@definition, [definition]),
        [definition_owner] = COALESCE(@definition_owner, [definition_owner]),
        [modified_date] = SYSUTCDATETIME()
    WHERE [id] = @id;
END;
