CREATE TABLE bill_items (
    id INTEGER PRIMARY KEY NOT NULL,
    bill_id INTEGER NOT NULL,
    item_template_id INTEGER NOT NULL,
    quantity REAL NOT NULL DEFAULT 1.0,
    note TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (bill_id) REFERENCES bills(id) ON DELETE CASCADE,
    FOREIGN KEY (item_template_id) REFERENCES item_templates(id)
);

-- Recreate bills without items column
CREATE TABLE bills_new (
    id INTEGER PRIMARY KEY NOT NULL,
    client_id INTEGER NOT NULL,
    date TEXT NOT NULL,
    due_date TEXT NOT NULL,
    reference TEXT NOT NULL,
    iban TEXT NOT NULL,
    notes TEXT NOT NULL,
    status TEXT NOT NULL,
    pdf_data BLOB,
    pdf_created_at TEXT,
    FOREIGN KEY (client_id) REFERENCES clients(id)
);
INSERT INTO bills_new SELECT id, client_id, date, due_date, reference, iban, notes, status, pdf_data, pdf_created_at FROM bills;
DROP TABLE bills;
ALTER TABLE bills_new RENAME TO bills;
