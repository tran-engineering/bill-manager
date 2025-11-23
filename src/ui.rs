use eframe::egui;
use chrono::Datelike;

use crate::app::{Bill, BillItem, BillManagerApp, BillStatus, Client, ItemTemplate, Tab};

impl eframe::App for BillManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Bill Manager");
                ui.separator();
                ui.selectable_value(&mut self.selected_tab, Tab::Bills, "Bills");
                ui.selectable_value(&mut self.selected_tab, Tab::Clients, "Clients");
                ui.selectable_value(&mut self.selected_tab, Tab::ItemTemplates, "Item Templates");
                ui.selectable_value(&mut self.selected_tab, Tab::Settings, "Settings");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.selected_tab {
                Tab::Clients => show_clients_tab(self, ui),
                Tab::Bills => show_bills_tab(self, ui),
                Tab::ItemTemplates => show_item_templates_tab(self, ui),
                Tab::Settings => show_settings_tab(self, ui),
            }
        });

        // Modal dialogs
        if self.show_client_form {
            show_client_form_window(self, ctx);
        }

        if self.show_bill_form {
            show_bill_form_window(self, ctx);
        }

        if self.show_template_form {
            show_template_form_window(self, ctx);
        }
    }
}

fn show_clients_tab(app: &mut BillManagerApp, ui: &mut egui::Ui) {
    ui.heading("Clients");
    ui.separator();

    if ui.button("➕ Add Client").clicked() {
        app.editing_client = Some(Client::default());
        app.show_client_form = true;
    }

    ui.add_space(10.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for client in app.clients.clone().iter() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.strong(&client.name);
                        ui.label(format!("{}, {}", client.address.city, client.address.country));
                        ui.label(&client.email);
                        ui.label(&client.phone);
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑 Delete").clicked() {
                            app.delete_client(client.id);
                        }
                        if ui.button("✏ Edit").clicked() {
                            app.editing_client = Some(client.clone());
                            app.show_client_form = true;
                        }
                    });
                });
            });
            ui.add_space(5.0);
        }
    });
}

fn show_bills_tab(app: &mut BillManagerApp, ui: &mut egui::Ui) {
    ui.heading("Bills");
    ui.separator();

    if ui.button("➕ Create Bill").clicked() {
        let mut new_bill = Bill::default();
        new_bill.iban = app.default_iban.clone();
        // Generate SCOR reference with next bill ID (temporary, will be updated on save)
        let year = chrono::Local::now().year();
        new_bill.reference = Bill::generate_scor_reference(app.next_bill_id, 0, year);
        app.editing_bill = Some(new_bill);
        app.show_bill_form = true;
    }

    ui.add_space(10.0);

    let mut bill_to_delete: Option<u64> = None;
    let mut bill_to_edit: Option<Bill> = None;
    let mut bill_to_generate_pdf: Option<u64> = None;
    let mut bill_to_save_pdf: Option<u64> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for bill in app.bills.clone().iter() {
            let client_name = app.get_client(bill.client_id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "Unknown Client".to_string());

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(format!("Bill #{}", bill.id));
                            ui.label("-");
                            ui.label(&client_name);
                        });
                        ui.label(format!("Date: {}", bill.date.format("%Y-%m-%d")));
                        ui.label(format!("Due: {}", bill.due_date.format("%Y-%m-%d")));
                        ui.label(format!("Total: CHF {:.2}", bill.total()));
                        ui.label(format!("Status: {}", bill.status));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑 Delete").clicked() {
                            bill_to_delete = Some(bill.id);
                        }
                        if ui.button("✏ Edit").clicked() {
                            bill_to_edit = Some(bill.clone());
                        }

                        // PDF button with green color if PDF exists
                        let pdf_exists = bill.pdf_data.is_some();
                        let pdf_button = if pdf_exists {
                            egui::Button::new("💾 Save PDF")
                                .fill(egui::Color32::from_rgb(60, 150, 60))
                        } else {
                            egui::Button::new("📄 Generate PDF")
                        };

                        if ui.add(pdf_button).clicked() {
                            if pdf_exists {
                                bill_to_save_pdf = Some(bill.id);
                            } else {
                                bill_to_generate_pdf = Some(bill.id);
                            }
                        }
                    });
                });
            });
            ui.add_space(5.0);
        }
    });

    if let Some(id) = bill_to_delete {
        app.delete_bill(id);
    }
    if let Some(bill) = bill_to_edit {
        app.editing_bill = Some(bill);
        app.show_bill_form = true;
    }
    if let Some(bill_id) = bill_to_generate_pdf {
        match app.generate_pdf(bill_id) {
            Ok(_) => {
                println!("PDF generated successfully");
            }
            Err(e) => {
                app.bill_error = Some(format!("Failed to generate PDF: {}", e));
            }
        }
    }
    if let Some(bill_id) = bill_to_save_pdf {
        match app.save_pdf_to_file(bill_id) {
            Ok(_) => {
                println!("PDF saved successfully");
            }
            Err(e) => {
                app.bill_error = Some(format!("Failed to save PDF: {}", e));
            }
        }
    }
}

fn show_settings_tab(app: &mut BillManagerApp, ui: &mut egui::Ui) {
    ui.heading("Settings");
    ui.separator();

    ui.add_space(10.0);

    let mut settings_changed = false;

    ui.group(|ui| {
        ui.strong("Your Business Information");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Company Name:");
            if ui.text_edit_singleline(&mut app.creditor_address.name).changed() {
                settings_changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Street:");
            let mut street = app.creditor_address.street.clone().unwrap_or_default();
            let response = ui.text_edit_singleline(&mut street);
            app.creditor_address.street = Some(street);
            if response.changed() {
                settings_changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Building Number:");
            let mut building = app.creditor_address.building_number.clone().unwrap_or_default();
            let response = ui.text_edit_singleline(&mut building);
            app.creditor_address.building_number = Some(building);
            if response.changed() {
                settings_changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Postal Code:");
            if ui.text_edit_singleline(&mut app.creditor_address.postal_code).changed() {
                settings_changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("City:");
            if ui.text_edit_singleline(&mut app.creditor_address.city).changed() {
                settings_changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Country:");
            if ui.text_edit_singleline(&mut app.creditor_address.country).changed() {
                settings_changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Default IBAN:");
            if ui.text_edit_singleline(&mut app.default_iban).changed() {
                settings_changed = true;
            }
        });
    });

    if settings_changed {
        app.save_settings();
    }
}

fn show_client_form_window(app: &mut BillManagerApp, ctx: &egui::Context) {
    let mut open = true;
    egui::Window::new("Client Details")
        .open(&mut open)
        .resizable(true)
        .show(ctx, |ui| {
            if let Some(client) = &mut app.editing_client {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut client.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Email:");
                    ui.text_edit_singleline(&mut client.email);
                });

                ui.horizontal(|ui| {
                    ui.label("Phone:");
                    ui.text_edit_singleline(&mut client.phone);
                });

                ui.separator();
                ui.strong("Address");

                ui.horizontal(|ui| {
                    ui.label("Street:");
                    let mut street = client.address.street.clone().unwrap_or_default();
                    ui.text_edit_singleline(&mut street);
                    client.address.street = Some(street);
                });

                ui.horizontal(|ui| {
                    ui.label("Building Number:");
                    let mut building = client.address.building_number.clone().unwrap_or_default();
                    ui.text_edit_singleline(&mut building);
                    client.address.building_number = Some(building);
                });

                ui.horizontal(|ui| {
                    ui.label("Postal Code:");
                    ui.text_edit_singleline(&mut client.address.postal_code);
                });

                ui.horizontal(|ui| {
                    ui.label("City:");
                    ui.text_edit_singleline(&mut client.address.city);
                });

                ui.horizontal(|ui| {
                    ui.label("Country:");
                    ui.text_edit_singleline(&mut client.address.country);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("💾 Save").clicked() {
                        let client = app.editing_client.take().unwrap();
                        if client.id == 0 {
                            app.add_client(client);
                        } else {
                            app.update_client(client);
                        }
                        app.show_client_form = false;
                    }

                    if ui.button("❌ Cancel").clicked() {
                        app.editing_client = None;
                        app.show_client_form = false;
                    }
                });
            }
        });

    if !open {
        app.editing_client = None;
        app.show_client_form = false;
    }
}

fn show_bill_form_window(app: &mut BillManagerApp, ctx: &egui::Context) {
    let mut open = true;
    let mut save_bill = false;
    let mut cancel_bill = false;

    // Get data before borrowing mutably
    let client_name = if let Some(bill) = &app.editing_bill {
        app.get_client(bill.client_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Select Client".to_string())
    } else {
        "Select Client".to_string()
    };
    let clients = app.clients.clone();
    let item_templates = app.item_templates.clone();

    egui::Window::new("Bill Details")
        .open(&mut open)
        .resizable(true)
        .default_width(600.0)
        .show(ctx, |ui| {
            if let Some(bill) = &mut app.editing_bill {

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Display error message if present
                    if let Some(error) = &app.bill_error {
                        ui.colored_label(egui::Color32::RED, error);
                        ui.separator();
                    }

                    ui.horizontal(|ui| {
                        ui.label("Client:");
                        egui::ComboBox::from_id_salt("client_select")
                            .selected_text(&client_name)
                            .show_ui(ui, |ui| {
                                for client in &clients {
                                    if ui.selectable_value(
                                        &mut bill.client_id,
                                        client.id,
                                        &client.name,
                                    ).clicked() {
                                        // Clear error when client is selected
                                        app.bill_error = None;
                                    }
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Bill Date:");
                        let date_str = bill.date.format("%Y-%m-%d").to_string();
                        ui.label(&date_str);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Due Date:");
                        let mut due_date_str = bill.due_date.format("%Y-%m-%d").to_string();
                        if ui.text_edit_singleline(&mut due_date_str).changed() {
                            // Try to parse the date
                            if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(&due_date_str, "%Y-%m-%d") {
                                bill.due_date = naive_date.and_hms_opt(0, 0, 0)
                                    .unwrap()
                                    .and_local_timezone(chrono::Local)
                                    .unwrap();
                            }
                        }
                        if ui.button("+7d").clicked() {
                            bill.due_date = bill.due_date + chrono::Duration::days(7);
                        }
                        if ui.button("+30d").clicked() {
                            bill.due_date = bill.due_date + chrono::Duration::days(30);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Reference:");
                        ui.text_edit_singleline(&mut bill.reference);
                        if ui.button("🔄 Generate").clicked() {
                            let year = chrono::Local::now().year();
                            let bill_id = if bill.id == 0 { app.next_bill_id } else { bill.id };
                            bill.reference = Bill::generate_scor_reference(bill_id, bill.client_id, year);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        egui::ComboBox::from_id_salt("status_select")
                            .selected_text(format!("{}", bill.status))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut bill.status, BillStatus::Draft, "Draft");
                                ui.selectable_value(&mut bill.status, BillStatus::Sent, "Sent");
                                ui.selectable_value(&mut bill.status, BillStatus::Paid, "Paid");
                                ui.selectable_value(
                                    &mut bill.status,
                                    BillStatus::Overdue,
                                    "Overdue",
                                );
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Notes:");
                        ui.text_edit_multiline(&mut bill.notes);
                    });

                    ui.separator();
                    ui.strong("Items");

                    let mut item_to_remove: Option<usize> = None;
                    let items_count = bill.items.len();

                    for (idx, item) in bill.items.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Description:");
                                ui.text_edit_singleline(&mut item.description);

                                // Add template button if templates exist
                                if !item_templates.is_empty() {
                                    egui::ComboBox::from_id_salt(format!("template_{}", idx))
                                        .selected_text("📋")
                                        .show_ui(ui, |ui| {
                                            for template in &item_templates {
                                                if ui.button(&template.description).clicked() {
                                                    item.description = template.description.clone();
                                                    item.unit_price = template.unit_price;
                                                }
                                            }
                                        });
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Quantity:");
                                ui.add(egui::DragValue::new(&mut item.quantity).speed(0.1));

                                ui.label("Unit Price:");
                                ui.add(egui::DragValue::new(&mut item.unit_price).speed(0.1));

                                ui.label(format!("Total: CHF {:.2}", item.total()));

                                if items_count > 1 && ui.button("🗑").clicked() {
                                    item_to_remove = Some(idx);
                                }
                            });
                        });
                    }

                    if let Some(idx) = item_to_remove {
                        bill.items.remove(idx);
                    }

                    ui.horizontal(|ui| {
                        if ui.button("➕ Add Item").clicked() {
                            bill.items.push(BillItem::default());
                        }

                        if !item_templates.is_empty() {
                            egui::ComboBox::from_id_salt("add_from_template")
                                .selected_text("📋 Add from Template")
                                .show_ui(ui, |ui| {
                                    for template in &item_templates {
                                        if ui.button(&template.description).clicked() {
                                            bill.items.push(template.to_bill_item());
                                        }
                                    }
                                });
                        }
                    });

                    ui.separator();
                    ui.strong(format!("Total: CHF {:.2}", bill.total()));

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            save_bill = true;
                        }

                        if ui.button("❌ Cancel").clicked() {
                            cancel_bill = true;
                        }
                    });
                });
            }
        });

    if save_bill {
        let bill = app.editing_bill.as_ref().unwrap();

        // Validate that a client is selected
        if bill.client_id == 0 || app.get_client(bill.client_id).is_none() {
            // Show error message - keep the bill form open
            app.bill_error = Some("Please select a client before saving the bill.".to_string());
        } else {
            // Valid client selected, proceed with save
            app.bill_error = None;
            let bill = app.editing_bill.take().unwrap();
            if bill.id == 0 {
                app.add_bill(bill);
            } else {
                app.update_bill(bill);
            }
            app.show_bill_form = false;
        }
    }

    if cancel_bill || !open {
        app.editing_bill = None;
        app.show_bill_form = false;
        app.bill_error = None;
    }
}

fn show_item_templates_tab(app: &mut BillManagerApp, ui: &mut egui::Ui) {
    ui.heading("Item Templates");
    ui.separator();

    if ui.button("➕ Add Template").clicked() {
        app.editing_template = Some(ItemTemplate::default());
        app.show_template_form = true;
    }

    ui.add_space(10.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for template in app.item_templates.clone().iter() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.strong(&template.description);
                        ui.label(format!("CHF {:.2}", template.unit_price));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑 Delete").clicked() {
                            app.delete_item_template(template.id);
                        }
                        if ui.button("✏ Edit").clicked() {
                            app.editing_template = Some(template.clone());
                            app.show_template_form = true;
                        }
                    });
                });
            });
            ui.add_space(5.0);
        }
    });
}

fn show_template_form_window(app: &mut BillManagerApp, ctx: &egui::Context) {
    let mut open = true;
    egui::Window::new("Item Template")
        .open(&mut open)
        .resizable(true)
        .show(ctx, |ui| {
            if let Some(template) = &mut app.editing_template {
                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.text_edit_singleline(&mut template.description);
                });

                ui.horizontal(|ui| {
                    ui.label("Unit Price:");
                    ui.add(egui::DragValue::new(&mut template.unit_price).speed(0.1).prefix("CHF "));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("💾 Save").clicked() {
                        let template = app.editing_template.take().unwrap();
                        if template.id == 0 {
                            app.add_item_template(template);
                        } else {
                            app.update_item_template(template);
                        }
                        app.show_template_form = false;
                    }

                    if ui.button("❌ Cancel").clicked() {
                        app.editing_template = None;
                        app.show_template_form = false;
                    }
                });
            }
        });

    if !open {
        app.editing_template = None;
        app.show_template_form = false;
    }
}
