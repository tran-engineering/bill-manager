use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Address {
    pub name: String,
    pub street: Option<String>,
    pub building_number: Option<String>,
    pub postal_code: String,
    pub city: String,
    pub country: String,
}

impl Address {
    pub fn new(
        name: String,
        street: Option<String>,
        building_number: Option<String>,
        postal_code: String,
        city: String,
        country: String,
    ) -> Self {
        Self {
            name,
            street,
            building_number,
            postal_code,
            city,
            country,
        }
    }
}
