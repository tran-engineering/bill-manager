use std::collections::HashMap;
use std::fs;
use std::io;
use std::process::Command;

use crate::app::{Bill, Client};
use crate::types::Address;

use text_placeholder::Template;

pub fn generate_bill_pdf(
    bill: &Bill,
    client: &Client,
    creditor: &Address,
) -> Result<Vec<u8>, String> {
    // Create Typst document content
    let typst_content = create_typst_invoice(bill, client, creditor);
    // Create a SystemWorld (loads fonts, stdlib, etc.)
    fs::write("temp.typ", typst_content).unwrap();

    Command::new("typst")
        .args(["compile", "temp.typ", "output.pdf"])
        .status()
        .unwrap();

    let pdf_data = read_file_as_bytes("output.pdf").unwrap();

    let _ = fs::remove_file("temp.typ");

    Ok(pdf_data)
}

fn read_file_as_bytes(path: &str) -> io::Result<Vec<u8>> {
    fs::read(path)
}

fn create_typst_invoice(bill: &Bill, client: &Client, creditor: &Address) -> String {
    let template_str = fs::read_to_string("templates/qr_bill.tpl").unwrap();

    let tpl = Template::new(&template_str);

    let amount_str = bill.total().to_string();

    let vars = HashMap::from([
        ("account", bill.iban.as_str()),
        ("creditor-name", creditor.name.as_str()),
        ("creditor-street", creditor.street.as_deref().unwrap_or("")),
        ("creditor-building", creditor.building_number.as_deref().unwrap_or("")),
        ("creditor-postal-code", creditor.postal_code.as_str()),
        ("creditor-city", creditor.city.as_str()),
        ("creditor-country", creditor.country.as_str()),
        ("amount", amount_str.as_str()),
        ("currency", "CHF"),
        ("debtor-name", client.name.as_str()),
        ("debtor-street", client.address.street.as_deref().unwrap_or("")),
        ("debtor-building", client.address.building_number.as_deref().unwrap_or("")),
        ("debtor-postal-code", client.address.postal_code.as_str()),
        ("debtor-city", client.address.city.as_str()),
        ("debtor-country", client.address.country.as_str()),
        ("reference-type", "SCOR"),
        ("reference", bill.reference.as_str()),
        ("additional-info", bill.notes.as_str()),
    ]);

    tpl.fill_with_hashmap(&vars)
}

pub fn save_bill_pdf(
    bill: &Bill,
    client: &Client,
    creditor: &Address,
    filename: &str,
) -> Result<(), String> {
    let pdf_data = generate_bill_pdf(bill, client, creditor)?;

    fs::write(filename, pdf_data)
        .map_err(|e| format!("Failed to write PDF file: {}", e))?;

    Ok(())
}
