DROP TABLE bill_items;

CREATE TABLE bills_new (
    id INTEGER PRIMARY KEY NOT NULL,
    client_id INTEGER NOT NULL,
    date TEXT NOT NULL,
    due_date TEXT NOT NULL,
    reference TEXT NOT NULL,
    iban TEXT NOT NULL,
    notes TEXT NOT NULL,
    status TEXT NOT NULL,
    items TEXT NOT NULL DEFAULT '[]',
    pdf_data BLOB,
    pdf_created_at TEXT,
    FOREIGN KEY (client_id) REFERENCES clients(id)
);
INSERT INTO bills_new SELECT id, client_id, date, due_date, reference, iban, notes, status, '[]', pdf_data, pdf_created_at FROM bills;
DROP TABLE bills;
ALTER TABLE bills_new RENAME TO bills;
