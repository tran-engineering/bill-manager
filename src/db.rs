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
                address_country TEXT NOT NULL
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
                FOREIGN KEY (client_id) REFERENCES clients(id)
            );

            CREATE TABLE IF NOT EXISTS item_templates (
                id INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                unit_price REAL NOT NULL
            );
            "#,
        )?;
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
                 address_postal_code, address_city, address_country)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
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
                ],
            )?;
            Ok(self.conn.last_insert_rowid() as u64)
        } else {
            // Update existing client
            self.conn.execute(
                r#"UPDATE clients SET
                name = ?1, email = ?2, phone = ?3, address_name = ?4,
                address_street = ?5, address_building_number = ?6,
                address_postal_code = ?7, address_city = ?8, address_country = ?9
                WHERE id = ?10"#,
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
                    client.id,
                ],
            )?;
            Ok(client.id)
        }
    }

    pub fn get_all_clients(&self) -> Result<Vec<Client>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, name, email, phone, address_name, address_street,
               address_building_number, address_postal_code, address_city, address_country
               FROM clients ORDER BY name"#,
        )?;

        let clients = stmt
            .query_map([], |row| {
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
                (client_id, date, due_date, reference, iban, notes, status, items)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
                params![
                    bill.client_id,
                    bill.date.to_rfc3339(),
                    bill.due_date.to_rfc3339(),
                    bill.reference,
                    bill.iban,
                    bill.notes,
                    status_str,
                    items_json,
                ],
            )?;
            Ok(self.conn.last_insert_rowid() as u64)
        } else {
            // Update existing bill
            self.conn.execute(
                r#"UPDATE bills SET
                client_id = ?1, date = ?2, due_date = ?3, reference = ?4,
                iban = ?5, notes = ?6, status = ?7, items = ?8
                WHERE id = ?9"#,
                params![
                    bill.client_id,
                    bill.date.to_rfc3339(),
                    bill.due_date.to_rfc3339(),
                    bill.reference,
                    bill.iban,
                    bill.notes,
                    status_str,
                    items_json,
                    bill.id,
                ],
            )?;
            Ok(bill.id)
        }
    }

    pub fn get_all_bills(&self) -> Result<Vec<Bill>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, client_id, date, due_date, reference, iban, notes, status, items
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
                r#"INSERT INTO item_templates (description, unit_price) VALUES (?1, ?2)"#,
                params![template.description, template.unit_price],
            )?;
            Ok(self.conn.last_insert_rowid() as u64)
        } else {
            // Update existing template
            self.conn.execute(
                r#"UPDATE item_templates SET description = ?1, unit_price = ?2 WHERE id = ?3"#,
                params![template.description, template.unit_price, template.id],
            )?;
            Ok(template.id)
        }
    }

    pub fn get_all_item_templates(&self) -> Result<Vec<ItemTemplate>> {
        let mut stmt = self
            .conn
            .prepare(r#"SELECT id, description, unit_price FROM item_templates ORDER BY description"#)?;

        let templates = stmt
            .query_map([], |row| {
                Ok(ItemTemplate {
                    id: row.get(0)?,
                    description: row.get(1)?,
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
