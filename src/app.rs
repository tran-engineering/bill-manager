use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use iso_11649::RfCreditorReference;
use iban::Iban;

use crate::db::Database;
use crate::types::Address;

pub fn validate_iban(iban_str: &str) -> bool {
    // Remove spaces and convert to uppercase for validation
    let cleaned = iban_str.replace(" ", "").to_uppercase();

    // Check if empty
    if cleaned.is_empty() {
        return false;
    }

    // Try to parse as IBAN
    cleaned.parse::<Iban>().is_ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: u64,
    pub name: String,
    pub address: Address,
    pub billing_address: Address,
    pub email: String,
    pub phone: String,
}

impl Default for Client {
    fn default() -> Self {
        let default_address = Address::new(
            String::new(),
            Some(String::new()),
            Some(String::new()),
            String::new(),
            String::new(),
            "CH".to_string(),
        );
        Self {
            id: 0,
            name: String::new(),
            address: default_address.clone(),
            billing_address: default_address,
            email: String::new(),
            phone: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTemplate {
    pub id: u64,
    pub item_type: String,
    pub unit_price: f64,
}

impl ItemTemplate {
    pub fn to_bill_item(&self) -> BillItem {
        BillItem {
            item_type: self.item_type.clone(),
            quantity: 1.0,
            unit_price: self.unit_price,
            note: String::new(),
        }
    }
}

impl Default for ItemTemplate {
    fn default() -> Self {
        Self {
            id: 0,
            item_type: String::new(),
            unit_price: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillItem {
    pub item_type: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub note: String,
}

impl BillItem {
    pub fn total(&self) -> f64 {
        self.quantity * self.unit_price
    }
}

impl Default for BillItem {
    fn default() -> Self {
        Self {
            item_type: String::new(),
            quantity: 1.0,
            unit_price: 0.0,
            note: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bill {
    pub id: u64,
    pub client_id: u64,
    pub date: DateTime<Local>,
    pub due_date: DateTime<Local>,
    pub items: Vec<BillItem>,
    pub reference: String,
    pub iban: String,
    pub notes: String,
    pub status: BillStatus,
    #[serde(skip)]
    pub pdf_data: Option<Vec<u8>>,
    pub pdf_created_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BillStatus {
    Draft,
    Sent,
    Paid,
    Overdue,
}

impl std::fmt::Display for BillStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BillStatus::Draft => write!(f, "Draft"),
            BillStatus::Sent => write!(f, "Sent"),
            BillStatus::Paid => write!(f, "Paid"),
            BillStatus::Overdue => write!(f, "Overdue"),
        }
    }
}

impl Bill {
    pub fn total(&self) -> f64 {
        self.items.iter().map(|item| item.total()).sum()
    }

    pub fn generate_scor_reference(bill_id: u64, client_id: u64, year: i32) -> String {
        
        // Format: YYYY-CCC-BBBB (year-client-bill) without separators for calculation
        // We'll add separators for display
        let base = format!("{:04}Y{:03}K{:04}", year, (client_id+420) % 1000, (bill_id+4200) % 10000);
        let rf = RfCreditorReference::new(base.as_str());
        

        // Calculate ISO 11649 check digits for SCOR
        rf.to_string()
    }
}

impl Default for Bill {
    fn default() -> Self {
        let now = Local::now();
        let due_date = now + chrono::Duration::days(30);

        Self {
            id: 0,
            client_id: 0,
            date: now,
            due_date,
            items: vec![BillItem::default()],
            reference: String::new(),
            iban: String::new(),
            notes: String::new(),
            status: BillStatus::Draft,
            pdf_data: None,
            pdf_created_at: None,
        }
    }
}

pub struct BillManagerApp {
    pub clients: Vec<Client>,
    pub bills: Vec<Bill>,
    pub item_templates: Vec<ItemTemplate>,
    pub next_client_id: u64,
    pub next_bill_id: u64,
    pub next_template_id: u64,

    // UI State
    pub selected_tab: Tab,
    pub editing_client: Option<Client>,
    pub editing_bill: Option<Bill>,
    pub editing_template: Option<ItemTemplate>,
    pub show_client_form: bool,
    pub show_bill_form: bool,
    pub show_template_form: bool,
    pub bill_error: Option<String>,

    // Creditor info (your business)
    pub creditor_address: Address,
    pub default_iban: String,

    // Database
    pub db: Arc<Mutex<Database>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Clients,
    Bills,
    ItemTemplates,
    Settings,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Bills
    }
}

impl BillManagerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize database
        let db = Database::new("bills.db").expect("Failed to open database");
        let db = Arc::new(Mutex::new(db));

        // Load data from database
        let clients = db.lock().unwrap().get_all_clients().unwrap_or_default();
        let bills = db.lock().unwrap().get_all_bills().unwrap_or_default();
        let item_templates = db.lock().unwrap().get_all_item_templates().unwrap_or_default();

        let next_client_id = db.lock().unwrap().get_next_client_id().unwrap_or(1);
        let next_bill_id = db.lock().unwrap().get_next_bill_id().unwrap_or(1);
        let next_template_id = db.lock().unwrap().get_next_template_id().unwrap_or(1);

        let creditor_address = db
            .lock()
            .unwrap()
            .get_creditor_address()
            .unwrap_or(None)
            .unwrap_or_else(|| Address::new(
                "Your Company Name".to_string(),
                Some("Your Street".to_string()),
                Some("1".to_string()),
                "8000".to_string(),
                "Zurich".to_string(),
                "CH".to_string(),
            ));

        let default_iban = db
            .lock()
            .unwrap()
            .get_default_iban()
            .unwrap_or(None)
            .unwrap_or_else(|| "CH93 0076 2011 6238 5295 7".to_string());

        Self {
            clients,
            bills,
            item_templates,
            next_client_id,
            next_bill_id,
            next_template_id,
            selected_tab: Tab::default(),
            editing_client: None,
            editing_bill: None,
            editing_template: None,
            show_client_form: false,
            show_bill_form: false,
            show_template_form: false,
            bill_error: None,
            creditor_address,
            default_iban,
            db,
        }
    }

    pub fn add_client(&mut self, mut client: Client) {
        let db = self.db.lock().unwrap();
        let id = db.save_client(&client).expect("Failed to save client");
        client.id = id;
        drop(db);

        self.clients.push(client);
        self.next_client_id = self.next_client_id.max(id + 1);
    }

    pub fn update_client(&mut self, client: Client) {
        let db = self.db.lock().unwrap();
        db.save_client(&client).expect("Failed to update client");
        drop(db);

        if let Some(pos) = self.clients.iter().position(|c| c.id == client.id) {
            self.clients[pos] = client;
        }
    }

    pub fn delete_client(&mut self, id: u64) {
        let db = self.db.lock().unwrap();
        db.delete_client(id).expect("Failed to delete client");
        drop(db);

        self.clients.retain(|c| c.id != id);
    }

    pub fn add_bill(&mut self, mut bill: Bill) {
        let db = self.db.lock().unwrap();
        let id = db.save_bill(&bill).expect("Failed to save bill");
        bill.id = id;
        drop(db);

        self.bills.push(bill);
        self.next_bill_id = self.next_bill_id.max(id + 1);
    }

    pub fn update_bill(&mut self, bill: Bill) {
        let db = self.db.lock().unwrap();
        db.save_bill(&bill).expect("Failed to update bill");
        drop(db);

        if let Some(pos) = self.bills.iter().position(|b| b.id == bill.id) {
            self.bills[pos] = bill;
        }
    }

    pub fn delete_bill(&mut self, id: u64) {
        let db = self.db.lock().unwrap();
        db.delete_bill(id).expect("Failed to delete bill");
        drop(db);

        self.bills.retain(|b| b.id != id);
    }

    pub fn save_settings(&self) {
        let db = self.db.lock().unwrap();
        db.save_creditor_address(&self.creditor_address)
            .expect("Failed to save creditor address");
        db.save_default_iban(&self.default_iban)
            .expect("Failed to save default IBAN");
    }

    pub fn get_client(&self, id: u64) -> Option<&Client> {
        self.clients.iter().find(|c| c.id == id)
    }

    pub fn add_item_template(&mut self, mut template: ItemTemplate) {
        let db = self.db.lock().unwrap();
        let id = db.save_item_template(&template).expect("Failed to save template");
        template.id = id;
        drop(db);

        self.item_templates.push(template);
        self.next_template_id = self.next_template_id.max(id + 1);
    }

    pub fn update_item_template(&mut self, template: ItemTemplate) {
        let db = self.db.lock().unwrap();
        db.save_item_template(&template).expect("Failed to update template");
        drop(db);

        if let Some(pos) = self.item_templates.iter().position(|t| t.id == template.id) {
            self.item_templates[pos] = template;
        }
    }

    pub fn delete_item_template(&mut self, id: u64) {
        let db = self.db.lock().unwrap();
        db.delete_item_template(id).expect("Failed to delete template");
        drop(db);

        self.item_templates.retain(|t| t.id != id);
    }

    pub fn generate_pdf(&mut self, bill_id: u64) -> Result<(), String> {
        let bill = self.bills.iter().find(|b| b.id == bill_id)
            .ok_or_else(|| "Bill not found".to_string())?;

        let client = self.get_client(bill.client_id)
            .ok_or_else(|| "Client not found".to_string())?;

        // Generate PDF in memory
        let pdf_data = crate::pdf::generate_bill_pdf(bill, client, &self.creditor_address)?;
        let now = Local::now();

        // Save to database
        let db = self.db.lock().unwrap();
        db.save_bill_pdf(bill_id, &pdf_data, &now)
            .map_err(|e| format!("Failed to save PDF to database: {}", e))?;
        drop(db);

        // Update bill in memory
        if let Some(bill) = self.bills.iter_mut().find(|b| b.id == bill_id) {
            bill.pdf_data = Some(pdf_data);
            bill.pdf_created_at = Some(now);
        }

        Ok(())
    }

    pub fn save_pdf_to_file(&self, bill_id: u64) -> Result<Option<std::path::PathBuf>, String> {
        // Use native file dialog
        let file_dialog = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .set_file_name(&format!("invoice_{}.pdf", bill_id));

        if let Some(path) = file_dialog.save_file() {
            let bill = self.bills.iter().find(|b| b.id == bill_id)
                .ok_or_else(|| "Bill not found".to_string())?;

            if let Some(pdf_data) = &bill.pdf_data {
                std::fs::write(&path, pdf_data)
                    .map_err(|e| format!("Failed to save PDF: {}", e))?;
                return Ok(Some(path));
            } else {
                return Err("PDF not generated yet".to_string());
            }
        }

        Ok(None)
    }

    pub fn update_bill_status(&mut self, bill_id: u64, new_status: BillStatus) {
        if let Some(bill) = self.bills.iter_mut().find(|b| b.id == bill_id) {
            bill.status = new_status;
            let db = self.db.lock().unwrap();
            db.save_bill(bill).ok();
        }
    }
}
