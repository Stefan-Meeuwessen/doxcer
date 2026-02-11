-- Azure SQL Database compatible DDL
-- Behavior:
-- - id is auto-generated via IDENTITY.
-- - creation_date is auto-generated on INSERT.
-- - modified_date is auto-generated on INSERT and refreshed on every UPDATE.

CREATE TABLE dbo.definitions
(
    [id] BIGINT IDENTITY(1,1) NOT NULL
        CONSTRAINT PK_definitions PRIMARY KEY CLUSTERED,

    [lakehouse] VARCHAR(256) NULL,
    [database] VARCHAR(256) NULL,
    [table] VARCHAR(256) NULL,
    [column] VARCHAR(256) NULL,

    [dashboard] VARCHAR(512) NULL,
    [definition] VARCHAR(8000) NULL,
    [definition_owner] VARCHAR(256) NULL,

    [creation_date] DATETIME2(3) NOT NULL
        CONSTRAINT DF_definitions_creation_date DEFAULT SYSUTCDATETIME(),
    [modified_date] DATETIME2(3) NOT NULL
        CONSTRAINT DF_definitions_modified_date DEFAULT SYSUTCDATETIME()
);
GO

CREATE OR ALTER TRIGGER dbo.trg_definitions_set_modified_date
ON dbo.definitions
AFTER UPDATE
AS
BEGIN
    SET NOCOUNT ON;

    -- Prevent nested trigger recursion on the internal UPDATE below.
    IF TRIGGER_NESTLEVEL() > 1
    BEGIN
        RETURN;
    END;

    UPDATE d
    SET d.[modified_date] = SYSUTCDATETIME()
    FROM dbo.definitions AS d
    INNER JOIN inserted AS i
        ON d.[id] = i.[id];
END;
GO
