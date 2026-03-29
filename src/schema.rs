// @generated automatically by Diesel CLI.

diesel::table! {
    bill_items (id) {
        id -> Integer,
        bill_id -> Integer,
        item_template_id -> Integer,
        quantity -> Double,
        note -> Text,
    }
}

diesel::table! {
    bills (id) {
        id -> Integer,
        client_id -> Integer,
        date -> Text,
        due_date -> Text,
        reference -> Text,
        iban -> Text,
        notes -> Text,
        status -> Text,
        pdf_data -> Nullable<Binary>,
        pdf_created_at -> Nullable<Text>,
    }
}

diesel::table! {
    clients (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        phone -> Text,
        address_name -> Text,
        address_street -> Nullable<Text>,
        address_building_number -> Nullable<Text>,
        address_postal_code -> Text,
        address_city -> Text,
        address_country -> Text,
        billing_address_name -> Nullable<Text>,
        billing_address_street -> Nullable<Text>,
        billing_address_building_number -> Nullable<Text>,
        billing_address_postal_code -> Nullable<Text>,
        billing_address_city -> Nullable<Text>,
        billing_address_country -> Nullable<Text>,
    }
}

diesel::table! {
    item_templates (id) {
        id -> Integer,
        item_type -> Text,
        unit_price -> Double,
        unit -> Text,
    }
}

diesel::table! {
    settings (key) {
        key -> Text,
        value -> Text,
    }
}

diesel::joinable!(bills -> clients (client_id));
diesel::joinable!(bill_items -> bills (bill_id));
diesel::joinable!(bill_items -> item_templates (item_template_id));

diesel::allow_tables_to_appear_in_same_query!(
    bill_items,
    bills,
    clients,
    item_templates,
    settings,
);
