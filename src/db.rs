use rusqlite::{params, Connection, Result};
use serde_json;

use crate::app::{Bill, BillItem, BillStatus, Client, ItemTemplate};
use crate::types::Address;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS clients (
                id INTEGER PRIMARY KEY,
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
                id INTEGER PRIMARY KEY,
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
                id INTEGER PRIMARY KEY,
                item_type TEXT NOT NULL,
                unit_price REAL NOT NULL
            );
            "#,
        )?;

        // Migration: Add pdf_data column if it doesn't exist
        self.conn.execute_batch(
            r#"
            -- Check if pdf_data column exists, if not add it
            PRAGMA table_info(bills);
            "#,
        ).ok();

        // Try to add the column (will fail silently if it already exists)
        self.conn.execute(
            "ALTER TABLE bills ADD COLUMN pdf_data BLOB",
            [],
        ).ok();

        self.conn.execute(
            "ALTER TABLE bills ADD COLUMN pdf_created_at TEXT",
            [],
        ).ok();

        // Add billing address columns if they don't exist
        self.conn.execute(
            "ALTER TABLE clients ADD COLUMN billing_address_name TEXT",
            [],
        ).ok();

        self.conn.execute(
            "ALTER TABLE clients ADD COLUMN billing_address_street TEXT",
            [],
        ).ok();

        self.conn.execute(
            "ALTER TABLE clients ADD COLUMN billing_address_building_number TEXT",
            [],
        ).ok();

        self.conn.execute(
            "ALTER TABLE clients ADD COLUMN billing_address_postal_code TEXT",
            [],
        ).ok();

        self.conn.execute(
            "ALTER TABLE clients ADD COLUMN billing_address_city TEXT",
            [],
        ).ok();

        self.conn.execute(
            "ALTER TABLE clients ADD COLUMN billing_address_country TEXT",
            [],
        ).ok();

        // Migration: Rename description column to item_type in item_templates
        // SQLite doesn't support RENAME COLUMN directly in older versions, so we check if the column exists
        let has_description = self.conn
            .prepare("SELECT description FROM item_templates LIMIT 1")
            .is_ok();

        if has_description {
            // Create new table with correct schema
            self.conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS item_templates_new (
                    id INTEGER PRIMARY KEY,
                    item_type TEXT NOT NULL,
                    unit_price REAL NOT NULL
                );
                INSERT INTO item_templates_new (id, item_type, unit_price)
                SELECT id, description, unit_price FROM item_templates;
                DROP TABLE item_templates;
                ALTER TABLE item_templates_new RENAME TO item_templates;
                "#,
            ).ok();
        }

        Ok(())
    }

    // Settings operations
    pub fn save_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            let value: String = row.get(0)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn save_creditor_address(&self, address: &Address) -> Result<()> {
        let json = serde_json::to_string(address).unwrap();
        self.save_setting("creditor_address", &json)
    }

    pub fn get_creditor_address(&self) -> Result<Option<Address>> {
        if let Some(json) = self.get_setting("creditor_address")? {
            let address: Address = serde_json::from_str(&json).unwrap();
            Ok(Some(address))
        } else {
            Ok(None)
        }
    }

    pub fn save_default_iban(&self, iban: &str) -> Result<()> {
        self.save_setting("default_iban", iban)
    }

    pub fn get_default_iban(&self) -> Result<Option<String>> {
        self.get_setting("default_iban")
    }

    // Client operations
    pub fn save_client(&self, client: &Client) -> Result<u64> {
        if client.id == 0 {
            // Insert new client
            self.conn.execute(
                r#"INSERT INTO clients
                (name, email, phone, address_name, address_street, address_building_number,
                 address_postal_code, address_city, address_country,
                 billing_address_name, billing_address_street, billing_address_building_number,
                 billing_address_postal_code, billing_address_city, billing_address_country)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)"#,
                params![
                    client.name,
                    client.email,
                    client.phone,
                    client.address.name,
                    client.address.street,
                    client.address.building_number,
                    client.address.postal_code,
                    client.address.city,
                    client.address.country,
                    client.billing_address.name,
                    client.billing_address.street,
                    client.billing_address.building_number,
                    client.billing_address.postal_code,
                    client.billing_address.city,
                    client.billing_address.country,
                ],
            )?;
            Ok(self.conn.last_insert_rowid() as u64)
        } else {
            // Update existing client
            self.conn.execute(
                r#"UPDATE clients SET
                name = ?1, email = ?2, phone = ?3, address_name = ?4,
                address_street = ?5, address_building_number = ?6,
                address_postal_code = ?7, address_city = ?8, address_country = ?9,
                billing_address_name = ?10, billing_address_street = ?11,
                billing_address_building_number = ?12, billing_address_postal_code = ?13,
                billing_address_city = ?14, billing_address_country = ?15
                WHERE id = ?16"#,
                params![
                    client.name,
                    client.email,
                    client.phone,
                    client.address.name,
                    client.address.street,
                    client.address.building_number,
                    client.address.postal_code,
                    client.address.city,
                    client.address.country,
                    client.billing_address.name,
                    client.billing_address.street,
                    client.billing_address.building_number,
                    client.billing_address.postal_code,
                    client.billing_address.city,
                    client.billing_address.country,
                    client.id,
                ],
            )?;
            Ok(client.id)
        }
    }

    pub fn get_all_clients(&self) -> Result<Vec<Client>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, name, email, phone, address_name, address_street,
               address_building_number, address_postal_code, address_city, address_country,
               billing_address_name, billing_address_street, billing_address_building_number,
               billing_address_postal_code, billing_address_city, billing_address_country
               FROM clients ORDER BY name"#,
        )?;

        let clients = stmt
            .query_map([], |row| {
                // For backward compatibility, if billing address is NULL, use regular address
                let billing_name: Option<String> = row.get(10)?;
                let billing_postal: Option<String> = row.get(13)?;
                let billing_city: Option<String> = row.get(14)?;
                let billing_country: Option<String> = row.get(15)?;

                let billing_address = if billing_name.is_some() && billing_postal.is_some() && billing_city.is_some() && billing_country.is_some() {
                    Address {
                        name: billing_name.unwrap(),
                        street: row.get(11)?,
                        building_number: row.get(12)?,
                        postal_code: billing_postal.unwrap(),
                        city: billing_city.unwrap(),
                        country: billing_country.unwrap(),
                    }
                } else {
                    // Use regular address as billing address for backward compatibility
                    Address {
                        name: row.get(4)?,
                        street: row.get(5)?,
                        building_number: row.get(6)?,
                        postal_code: row.get(7)?,
                        city: row.get(8)?,
                        country: row.get(9)?,
                    }
                };

                Ok(Client {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    email: row.get(2)?,
                    phone: row.get(3)?,
                    address: Address {
                        name: row.get(4)?,
                        street: row.get(5)?,
                        building_number: row.get(6)?,
                        postal_code: row.get(7)?,
                        city: row.get(8)?,
                        country: row.get(9)?,
                    },
                    billing_address,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(clients)
    }

    pub fn delete_client(&self, id: u64) -> Result<()> {
        self.conn
            .execute("DELETE FROM clients WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_next_client_id(&self) -> Result<u64> {
        let mut stmt = self.conn.prepare("SELECT COALESCE(MAX(id), 0) + 1 FROM clients")?;
        let next_id: u64 = stmt.query_row([], |row| row.get(0))?;
        Ok(next_id)
    }

    // Bill operations
    pub fn save_bill(&self, bill: &Bill) -> Result<u64> {
        let items_json = serde_json::to_string(&bill.items).unwrap();
        let status_str = match bill.status {
            BillStatus::Draft => "Draft",
            BillStatus::Sent => "Sent",
            BillStatus::Paid => "Paid",
            BillStatus::Overdue => "Overdue",
        };

        if bill.id == 0 {
            // Insert new bill
            self.conn.execute(
                r#"INSERT INTO bills
                (client_id, date, due_date, reference, iban, notes, status, items, pdf_data, pdf_created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
                params![
                    bill.client_id,
                    bill.date.to_rfc3339(),
                    bill.due_date.to_rfc3339(),
                    bill.reference,
                    bill.iban,
                    bill.notes,
                    status_str,
                    items_json,
                    bill.pdf_data.as_ref().map(|v| v.as_slice()),
                    bill.pdf_created_at.as_ref().map(|dt| dt.to_rfc3339()),
                ],
            )?;
            Ok(self.conn.last_insert_rowid() as u64)
        } else {
            // Update existing bill
            self.conn.execute(
                r#"UPDATE bills SET
                client_id = ?1, date = ?2, due_date = ?3, reference = ?4,
                iban = ?5, notes = ?6, status = ?7, items = ?8, pdf_data = ?9, pdf_created_at = ?10
                WHERE id = ?11"#,
                params![
                    bill.client_id,
                    bill.date.to_rfc3339(),
                    bill.due_date.to_rfc3339(),
                    bill.reference,
                    bill.iban,
                    bill.notes,
                    status_str,
                    items_json,
                    bill.pdf_data.as_ref().map(|v| v.as_slice()),
                    bill.pdf_created_at.as_ref().map(|dt| dt.to_rfc3339()),
                    bill.id,
                ],
            )?;
            Ok(bill.id)
        }
    }

    pub fn save_bill_pdf(&self, bill_id: u64, pdf_data: &[u8], created_at: &chrono::DateTime<chrono::Local>) -> Result<()> {
        self.conn.execute(
            "UPDATE bills SET pdf_data = ?1, pdf_created_at = ?2 WHERE id = ?3",
            params![pdf_data, created_at.to_rfc3339(), bill_id],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_bill_pdf(&self, bill_id: u64) -> Result<Option<Vec<u8>>> {
        let mut stmt = self.conn.prepare("SELECT pdf_data FROM bills WHERE id = ?1")?;
        let mut rows = stmt.query(params![bill_id])?;

        if let Some(row) = rows.next()? {
            let pdf_data: Option<Vec<u8>> = row.get(0)?;
            Ok(pdf_data)
        } else {
            Ok(None)
        }
    }

    pub fn get_all_bills(&self) -> Result<Vec<Bill>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, client_id, date, due_date, reference, iban, notes, status, items, pdf_data, pdf_created_at
               FROM bills ORDER BY date DESC"#,
        )?;

        let bills = stmt
            .query_map([], |row| {
                let status_str: String = row.get(7)?;
                let status = match status_str.as_str() {
                    "Draft" => BillStatus::Draft,
                    "Sent" => BillStatus::Sent,
                    "Paid" => BillStatus::Paid,
                    "Overdue" => BillStatus::Overdue,
                    _ => BillStatus::Draft,
                };

                let items_json: String = row.get(8)?;
                let items: Vec<BillItem> = serde_json::from_str(&items_json).unwrap();

                let date_str: String = row.get(2)?;
                let due_date_str: String = row.get(3)?;

                let pdf_created_at: Option<String> = row.get(10)?;
                let pdf_created_at = pdf_created_at.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Local))
                });

                Ok(Bill {
                    id: row.get(0)?,
                    client_id: row.get(1)?,
                    date: chrono::DateTime::parse_from_rfc3339(&date_str)
                        .unwrap()
                        .with_timezone(&chrono::Local),
                    due_date: chrono::DateTime::parse_from_rfc3339(&due_date_str)
                        .unwrap()
                        .with_timezone(&chrono::Local),
                    reference: row.get(4)?,
                    iban: row.get(5)?,
                    notes: row.get(6)?,
                    status,
                    items,
                    pdf_data: row.get(9)?,
                    pdf_created_at,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(bills)
    }

    pub fn delete_bill(&self, id: u64) -> Result<()> {
        self.conn
            .execute("DELETE FROM bills WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_next_bill_id(&self) -> Result<u64> {
        let mut stmt = self.conn.prepare("SELECT COALESCE(MAX(id), 0) + 1 FROM bills")?;
        let next_id: u64 = stmt.query_row([], |row| row.get(0))?;
        Ok(next_id)
    }

    // Item template operations
    pub fn save_item_template(&self, template: &ItemTemplate) -> Result<u64> {
        if template.id == 0 {
            // Insert new template
            self.conn.execute(
                r#"INSERT INTO item_templates (item_type, unit_price) VALUES (?1, ?2)"#,
                params![template.item_type, template.unit_price],
            )?;
            Ok(self.conn.last_insert_rowid() as u64)
        } else {
            // Update existing template
            self.conn.execute(
                r#"UPDATE item_templates SET item_type = ?1, unit_price = ?2 WHERE id = ?3"#,
                params![template.item_type, template.unit_price, template.id],
            )?;
            Ok(template.id)
        }
    }

    pub fn get_all_item_templates(&self) -> Result<Vec<ItemTemplate>> {
        let mut stmt = self
            .conn
            .prepare(r#"SELECT id, item_type, unit_price FROM item_templates ORDER BY item_type"#)?;

        let templates = stmt
            .query_map([], |row| {
                Ok(ItemTemplate {
                    id: row.get(0)?,
                    item_type: row.get(1)?,
                    unit_price: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(templates)
    }

    pub fn delete_item_template(&self, id: u64) -> Result<()> {
        self.conn
            .execute("DELETE FROM item_templates WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_next_template_id(&self) -> Result<u64> {
        let mut stmt = self
            .conn
            .prepare("SELECT COALESCE(MAX(id), 0) + 1 FROM item_templates")?;
        let next_id: u64 = stmt.query_row([], |row| row.get(0))?;
        Ok(next_id)
    }
}
