use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::error::Error;

use crate::app::{Bill, BillItem, BillStatus, Client, ItemTemplate};
use crate::models::*;
use crate::schema::*;
use crate::types::Address;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct Database {
    pool: DbPool,
}

impl Database {
    pub fn new(database_url: &str) -> Result<Self, Box<dyn Error>> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)?;

        // Run migrations
        let mut conn = pool.get()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| format!("Failed to run migrations: {}", e))?;

        Ok(Database { pool })
    }

    fn get_conn(&self) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, Box<dyn Error>> {
        Ok(self.pool.get()?)
    }

    // Settings operations
    pub fn save_setting(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        diesel::replace_into(settings::table)
            .values(&Setting {
                key: key.to_string(),
                value: value.to_string(),
            })
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        let result = settings::table
            .filter(settings::key.eq(key))
            .select(settings::value)
            .first::<String>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn save_creditor_address(&self, address: &Address) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string(address)?;
        self.save_setting("creditor_address", &json)
    }

    pub fn get_creditor_address(&self) -> Result<Option<Address>, Box<dyn Error>> {
        if let Some(json) = self.get_setting("creditor_address")? {
            let address: Address = serde_json::from_str(&json)?;
            Ok(Some(address))
        } else {
            Ok(None)
        }
    }

    pub fn save_default_iban(&self, iban: &str) -> Result<(), Box<dyn Error>> {
        self.save_setting("default_iban", iban)
    }

    pub fn get_default_iban(&self) -> Result<Option<String>, Box<dyn Error>> {
        self.get_setting("default_iban")
    }

    // Client operations
    pub fn save_client(&self, client: &Client) -> Result<u64, Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        if client.id == 0 {
            // Insert new client
            let new_client = NewClient {
                name: client.name.clone(),
                email: client.email.clone(),
                phone: client.phone.clone(),
                address_name: client.address.name.clone(),
                address_street: client.address.street.clone(),
                address_building_number: client.address.building_number.clone(),
                address_postal_code: client.address.postal_code.clone(),
                address_city: client.address.city.clone(),
                address_country: client.address.country.clone(),
                billing_address_name: Some(client.billing_address.name.clone()),
                billing_address_street: client.billing_address.street.clone(),
                billing_address_building_number: client.billing_address.building_number.clone(),
                billing_address_postal_code: Some(client.billing_address.postal_code.clone()),
                billing_address_city: Some(client.billing_address.city.clone()),
                billing_address_country: Some(client.billing_address.country.clone()),
            };

            let id = diesel::insert_into(clients::table)
                .values(&new_client)
                .returning(clients::id)
                .get_result::<i32>(&mut conn)?;

            Ok(id as u64)
        } else {
            // Update existing client
            let client_db = ClientDb {
                id: client.id as i32,
                name: client.name.clone(),
                email: client.email.clone(),
                phone: client.phone.clone(),
                address_name: client.address.name.clone(),
                address_street: client.address.street.clone(),
                address_building_number: client.address.building_number.clone(),
                address_postal_code: client.address.postal_code.clone(),
                address_city: client.address.city.clone(),
                address_country: client.address.country.clone(),
                billing_address_name: Some(client.billing_address.name.clone()),
                billing_address_street: client.billing_address.street.clone(),
                billing_address_building_number: client.billing_address.building_number.clone(),
                billing_address_postal_code: Some(client.billing_address.postal_code.clone()),
                billing_address_city: Some(client.billing_address.city.clone()),
                billing_address_country: Some(client.billing_address.country.clone()),
            };

            diesel::update(clients::table.filter(clients::id.eq(client.id as i32)))
                .set(&client_db)
                .execute(&mut conn)?;

            Ok(client.id)
        }
    }

    pub fn get_all_clients(&self) -> Result<Vec<Client>, Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        let clients_db: Vec<ClientDb> = clients::table
            .order(clients::name.asc())
            .load::<ClientDb>(&mut conn)?;

        let clients = clients_db.into_iter().map(|c| {
            let billing_address = if c.billing_address_name.is_some()
                && c.billing_address_postal_code.is_some()
                && c.billing_address_city.is_some()
                && c.billing_address_country.is_some() {
                Address {
                    name: c.billing_address_name.unwrap(),
                    street: c.billing_address_street.clone(),
                    building_number: c.billing_address_building_number.clone(),
                    postal_code: c.billing_address_postal_code.unwrap(),
                    city: c.billing_address_city.unwrap(),
                    country: c.billing_address_country.unwrap(),
                }
            } else {
                // Use regular address as billing address for backward compatibility
                Address {
                    name: c.address_name.clone(),
                    street: c.address_street.clone(),
                    building_number: c.address_building_number.clone(),
                    postal_code: c.address_postal_code.clone(),
                    city: c.address_city.clone(),
                    country: c.address_country.clone(),
                }
            };

            Client {
                id: c.id as u64,
                name: c.name,
                email: c.email,
                phone: c.phone,
                address: Address {
                    name: c.address_name,
                    street: c.address_street,
                    building_number: c.address_building_number,
                    postal_code: c.address_postal_code,
                    city: c.address_city,
                    country: c.address_country,
                },
                billing_address,
            }
        }).collect();

        Ok(clients)
    }

    pub fn delete_client(&self, id: u64) -> Result<(), Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        diesel::delete(clients::table.filter(clients::id.eq(id as i32)))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn get_next_client_id(&self) -> Result<u64, Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        let max_id: Option<Option<i32>> = clients::table
            .select(diesel::dsl::max(clients::id))
            .first(&mut conn)
            .optional()?;

        Ok((max_id.flatten().unwrap_or(0) + 1) as u64)
    }

    // Bill operations
    pub fn save_bill(&self, bill: &Bill) -> Result<u64, Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        let items_json = serde_json::to_string(&bill.items)?;
        let status_str = match bill.status {
            BillStatus::Draft => "Draft",
            BillStatus::Sent => "Sent",
            BillStatus::Paid => "Paid",
            BillStatus::Overdue => "Overdue",
        };

        if bill.id == 0 {
            // Insert new bill
            let new_bill = NewBill {
                client_id: bill.client_id as i32,
                date: bill.date.to_rfc3339(),
                due_date: bill.due_date.to_rfc3339(),
                reference: bill.reference.clone(),
                iban: bill.iban.clone(),
                notes: bill.notes.clone(),
                status: status_str.to_string(),
                items: items_json,
                pdf_data: bill.pdf_data.clone(),
                pdf_created_at: bill.pdf_created_at.as_ref().map(|dt| dt.to_rfc3339()),
            };

            let id = diesel::insert_into(bills::table)
                .values(&new_bill)
                .returning(bills::id)
                .get_result::<i32>(&mut conn)?;

            Ok(id as u64)
        } else {
            // Update existing bill
            let bill_db = BillDb {
                id: bill.id as i32,
                client_id: bill.client_id as i32,
                date: bill.date.to_rfc3339(),
                due_date: bill.due_date.to_rfc3339(),
                reference: bill.reference.clone(),
                iban: bill.iban.clone(),
                notes: bill.notes.clone(),
                status: status_str.to_string(),
                items: items_json,
                pdf_data: bill.pdf_data.clone(),
                pdf_created_at: bill.pdf_created_at.as_ref().map(|dt| dt.to_rfc3339()),
            };

            diesel::update(bills::table.filter(bills::id.eq(bill.id as i32)))
                .set(&bill_db)
                .execute(&mut conn)?;

            Ok(bill.id)
        }
    }

    pub fn save_bill_pdf(&self, bill_id: u64, pdf_data: &[u8], created_at: &chrono::DateTime<chrono::Local>) -> Result<(), Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        diesel::update(bills::table.filter(bills::id.eq(bill_id as i32)))
            .set((
                bills::pdf_data.eq(Some(pdf_data)),
                bills::pdf_created_at.eq(Some(created_at.to_rfc3339())),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn get_all_bills(&self) -> Result<Vec<Bill>, Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        let bills_db: Vec<BillDb> = bills::table
            .order(bills::date.desc())
            .load::<BillDb>(&mut conn)?;

        let bills = bills_db.into_iter().map(|b| {
            let status = match b.status.as_str() {
                "Draft" => BillStatus::Draft,
                "Sent" => BillStatus::Sent,
                "Paid" => BillStatus::Paid,
                "Overdue" => BillStatus::Overdue,
                _ => BillStatus::Draft,
            };

            let items: Vec<BillItem> = serde_json::from_str(&b.items).unwrap_or_default();

            let pdf_created_at = b.pdf_created_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Local))
            });

            Bill {
                id: b.id as u64,
                client_id: b.client_id as u64,
                date: chrono::DateTime::parse_from_rfc3339(&b.date)
                    .unwrap()
                    .with_timezone(&chrono::Local),
                due_date: chrono::DateTime::parse_from_rfc3339(&b.due_date)
                    .unwrap()
                    .with_timezone(&chrono::Local),
                reference: b.reference,
                iban: b.iban,
                notes: b.notes,
                status,
                items,
                pdf_data: b.pdf_data,
                pdf_created_at,
            }
        }).collect();

        Ok(bills)
    }

    pub fn delete_bill(&self, id: u64) -> Result<(), Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        diesel::delete(bills::table.filter(bills::id.eq(id as i32)))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn get_next_bill_id(&self) -> Result<u64, Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        let max_id: Option<Option<i32>> = bills::table
            .select(diesel::dsl::max(bills::id))
            .first(&mut conn)
            .optional()?;

        Ok((max_id.flatten().unwrap_or(0) + 1) as u64)
    }

    // Item template operations
    pub fn save_item_template(&self, template: &ItemTemplate) -> Result<u64, Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        if template.id == 0 {
            // Insert new template
            let new_template = NewItemTemplate {
                item_type: template.item_type.clone(),
                unit_price: template.unit_price,
            };

            let id = diesel::insert_into(item_templates::table)
                .values(&new_template)
                .returning(item_templates::id)
                .get_result::<i32>(&mut conn)?;

            Ok(id as u64)
        } else {
            // Update existing template
            let template_db = ItemTemplateDb {
                id: template.id as i32,
                item_type: template.item_type.clone(),
                unit_price: template.unit_price,
            };

            diesel::update(item_templates::table.filter(item_templates::id.eq(template.id as i32)))
                .set(&template_db)
                .execute(&mut conn)?;

            Ok(template.id)
        }
    }

    pub fn get_all_item_templates(&self) -> Result<Vec<ItemTemplate>, Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        let templates_db: Vec<ItemTemplateDb> = item_templates::table
            .order(item_templates::item_type.asc())
            .load::<ItemTemplateDb>(&mut conn)?;

        let templates = templates_db.into_iter().map(|t| {
            ItemTemplate {
                id: t.id as u64,
                item_type: t.item_type,
                unit_price: t.unit_price,
            }
        }).collect();

        Ok(templates)
    }

    pub fn delete_item_template(&self, id: u64) -> Result<(), Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        diesel::delete(item_templates::table.filter(item_templates::id.eq(id as i32)))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn get_next_template_id(&self) -> Result<u64, Box<dyn Error>> {
        let mut conn = self.get_conn()?;

        let max_id: Option<Option<i32>> = item_templates::table
            .select(diesel::dsl::max(item_templates::id))
            .first(&mut conn)
            .optional()?;

        Ok((max_id.flatten().unwrap_or(0) + 1) as u64)
    }
}
