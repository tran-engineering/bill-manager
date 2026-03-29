-- SQLite does not support DROP COLUMN in older versions; recreate table without the column
CREATE TABLE item_templates_backup (
    id INTEGER PRIMARY KEY NOT NULL,
    item_type TEXT NOT NULL,
    unit_price REAL NOT NULL
);
INSERT INTO item_templates_backup SELECT id, item_type, unit_price FROM item_templates;
DROP TABLE item_templates;
ALTER TABLE item_templates_backup RENAME TO item_templates;
