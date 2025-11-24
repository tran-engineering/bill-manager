CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS clients (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT NOT NULL,
    address_name TEXT NOT NULL,
    address_street TEXT,
    address_building_number TEXT,
    address_postal_code TEXT NOT NULL,
    address_city TEXT NOT NULL,
    address_country TEXT NOT NULL,
    billing_address_name TEXT,
    billing_address_street TEXT,
    billing_address_building_number TEXT,
    billing_address_postal_code TEXT,
    billing_address_city TEXT,
    billing_address_country TEXT
);

CREATE TABLE IF NOT EXISTS bills (
    id INTEGER PRIMARY KEY NOT NULL,
    client_id INTEGER NOT NULL,
    date TEXT NOT NULL,
    due_date TEXT NOT NULL,
    reference TEXT NOT NULL,
    iban TEXT NOT NULL,
    notes TEXT NOT NULL,
    status TEXT NOT NULL,
    items TEXT NOT NULL,
    pdf_data BLOB,
    pdf_created_at TEXT,
    FOREIGN KEY (client_id) REFERENCES clients(id)
);

CREATE TABLE IF NOT EXISTS item_templates (
    id INTEGER PRIMARY KEY NOT NULL,
    item_type TEXT NOT NULL,
    unit_price REAL NOT NULL
);
