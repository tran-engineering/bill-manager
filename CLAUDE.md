# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Swiss QR Bill Manager - A desktop application for managing invoices with Swiss QR bill generation, built with Rust, egui, and Diesel ORM.

## Build and Run Commands

```bash
# Build the application
cargo build

# Run the application
cargo run

# Check for compilation errors without building
cargo check

# Build for release
cargo build --release

# Run tests
cargo test
```

## Architecture

### Application Structure

**Three-layer architecture with in-memory caching:**

1. **Database Layer** (`src/db.rs`)
   - Diesel ORM with SQLite backend
   - Manages persistence for clients, bills, item templates, and settings
   - Connection pooling via `r2d2`
   - Database file: `bills.db` (created automatically)
   - All database methods return `Result<T, Box<dyn Error>>`

2. **Application State** (`src/app.rs`)
   - `BillManagerApp` holds both database reference AND in-memory caches
   - Caches must be kept in sync with database on all mutations
   - Pattern: Always update DB first, then update in-memory cache
   - Database access through `Arc<Mutex<Database>>` for thread-safety

3. **UI Layer** (`src/ui.rs`)
   - egui-based immediate mode GUI
   - UI reads from database via `app.get_bills()` (not from cache directly)
   - Uses deferred mutation pattern: collects actions during render, applies after

### Data Flow Pattern

**Read Operations:**
```rust
// UI should fetch fresh data from DB, not use app.bills directly
let bills = app.get_bills().unwrap_or_default();
```

**Write Operations:**
```rust
// 1. Write to database
let db = self.db.lock().unwrap();
db.save_bill(&bill)?;
drop(db);

// 2. Update in-memory cache
if let Some(pos) = self.bills.iter().position(|b| b.id == bill.id) {
    self.bills[pos] = bill;
}
```

### Core Data Models

**Dual model pattern for each entity:**
- Application models (`app.rs`): `Client`, `Bill`, `ItemTemplate` - Used in UI and business logic
- Database models (`models.rs`): `ClientDb`, `BillDb`, `ItemTemplateDb` - Diesel representations
- Conversion happens in `db.rs` methods

**Bill lifecycle:**
1. Draft → Sent → Paid/Overdue (tracked via `BillStatus` enum)
2. PDF generation is separate from bill creation
3. PDFs stored as `Vec<u8>` in database, cached in `Bill.pdf_data`

### PDF Generation

**Two-step process using Typst:**

1. **Template rendering** (`src/pdf.rs`):
   - Uses Typst templates (expected in `templates/` directory)
   - `text_placeholder` crate for variable substitution
   - QR code generation using `qrcode` crate
   - Custom `TypstWorld` implementation for package management

2. **PDF compilation**:
   - Typst compiles template to PDF
   - Result stored in database via `save_bill_pdf()`
   - PDF data cached in bill object

**Package cache location:** `~/.cache/typst/packages/` (or system equivalent)

### Database Schema

**Key tables managed by Diesel:**
- `clients` - Customer information with separate billing addresses
- `bills` - Invoices with JSON-serialized line items
- `item_templates` - Reusable line item templates
- `settings` - Key-value store for app configuration (creditor address, default IBAN)

**Important:** Bill items are stored as JSON string in `bills.items` column, not normalized.

### Special Features

**SCOR Reference Generation:**
- ISO 11649 compliant creditor references
- Format: `RF` + check digits + custom reference string
- Generated via `Bill::generate_scor_reference()` using `iso_11649` crate

**IBAN Validation:**
- Uses `iban` crate for validation
- Supports formatted input (with spaces)
- Function: `validate_iban()` in `app.rs`

**Filename Sanitization:**
- `sanitize_filename()` handles cross-platform path safety
- Replaces problematic characters: `/ \ : * ? " < > |`
- Normalizes whitespace and removes consecutive underscores
- Used for PDF export filenames

## Important Patterns

### Mutex Handling
Always drop mutex guards explicitly before long operations:
```rust
let db = self.db.lock().unwrap();
let result = db.operation();
drop(db);  // Explicit drop before next operation
```

### UI State Management
egui uses immediate mode rendering - collect mutations during render pass, apply afterward:
```rust
let mut item_to_delete: Option<u64> = None;
// ... in render loop ...
if ui.button("Delete").clicked() {
    item_to_delete = Some(id);
}
// ... after render ...
if let Some(id) = item_to_delete {
    app.delete_item(id);
}
```

### Error Handling
- Database operations return `Result<T, Box<dyn Error>>`
- UI-facing methods return `Result<T, String>` for display
- Use `.map_err(|e| format!("Context: {}", e))` when converting

## Diesel Migrations

Migrations are embedded in binary via `embed_migrations!` macro. They run automatically on first database connection. Schema changes require:
1. Create migration: `diesel migration generate migration_name`
2. Rebuild to embed new migrations
3. Schema is auto-regenerated in `src/schema.rs`
