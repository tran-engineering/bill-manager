use diesel::prelude::*;
use crate::schema::*;

// Database models (for Diesel)

#[derive(Queryable, Insertable, AsChangeset, Selectable, Debug, Clone)]
#[diesel(table_name = settings)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone)]
#[diesel(table_name = clients)]
pub struct ClientDb {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub address_name: String,
    pub address_street: Option<String>,
    pub address_building_number: Option<String>,
    pub address_postal_code: String,
    pub address_city: String,
    pub address_country: String,
    pub billing_address_name: Option<String>,
    pub billing_address_street: Option<String>,
    pub billing_address_building_number: Option<String>,
    pub billing_address_postal_code: Option<String>,
    pub billing_address_city: Option<String>,
    pub billing_address_country: Option<String>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = clients)]
pub struct NewClient {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub address_name: String,
    pub address_street: Option<String>,
    pub address_building_number: Option<String>,
    pub address_postal_code: String,
    pub address_city: String,
    pub address_country: String,
    pub billing_address_name: Option<String>,
    pub billing_address_street: Option<String>,
    pub billing_address_building_number: Option<String>,
    pub billing_address_postal_code: Option<String>,
    pub billing_address_city: Option<String>,
    pub billing_address_country: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone)]
#[diesel(table_name = bills)]
pub struct BillDb {
    pub id: i32,
    pub client_id: i32,
    pub date: String,
    pub due_date: String,
    pub reference: String,
    pub iban: String,
    pub notes: String,
    pub status: String,
    pub items: String,
    pub pdf_data: Option<Vec<u8>>,
    pub pdf_created_at: Option<String>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = bills)]
pub struct NewBill {
    pub client_id: i32,
    pub date: String,
    pub due_date: String,
    pub reference: String,
    pub iban: String,
    pub notes: String,
    pub status: String,
    pub items: String,
    pub pdf_data: Option<Vec<u8>>,
    pub pdf_created_at: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone)]
#[diesel(table_name = item_templates)]
pub struct ItemTemplateDb {
    pub id: i32,
    pub item_type: String,
    pub unit_price: f64,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = item_templates)]
pub struct NewItemTemplate {
    pub item_type: String,
    pub unit_price: f64,
}
