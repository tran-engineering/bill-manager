# Swiss QR Bill Manager

A desktop application for managing invoices with Swiss QR bill generation, built with Rust, egui, and Diesel ORM.

## Features

- **Invoice Management**: Create, edit, and track invoices with draft, sent, paid, and overdue statuses
- **Client Database**: Store and manage customer information with billing addresses
- **Swiss QR Bill Generation**: Automatically generate ISO 20022 compliant QR bills
- **PDF Export**: Professional invoice PDFs using Typst templates
- **Item Templates**: Reusable line item templates for common services/products
- **IBAN Validation**: Built-in validation for Swiss and international IBANs
- **SCOR References**: ISO 11649 compliant creditor reference generation

## Requirements

- Rust 2024 edition or later
- SQLite (bundled via diesel)

## Installation

Clone the repository and build:

```bash
git clone <repository-url>
cd swiss-qr-bill
cargo build --release
```

The compiled binary will be located at `target/release/bill-manager`.

## Usage

### Running the Application

```bash
cargo run
```

Or run the compiled binary directly:

```bash
./target/release/bill-manager
```

### First Time Setup

1. Launch the application
2. Navigate to Settings
3. Configure your creditor information (name, address, IBAN)
4. Set default values for new invoices

### Creating an Invoice

1. Go to the Bills tab
2. Click "New Bill"
3. Select a client (or create a new one)
4. Add line items manually or use templates
5. Generate the PDF with Swiss QR bill
6. Export or mark as sent

## Development

### Project Structure

```
src/
├── main.rs       # Application entry point
├── app.rs        # Application state and business logic
├── db.rs         # Database layer (Diesel ORM)
├── models.rs     # Database models
├── ui.rs         # egui UI implementation
├── pdf.rs        # PDF and QR bill generation
├── types.rs      # Core data types
└── schema.rs     # Auto-generated Diesel schema
```

### Architecture

The application follows a three-layer architecture:

1. **Database Layer** (`db.rs`): Diesel ORM with SQLite for persistence
2. **Application State** (`app.rs`): In-memory caching with database synchronization
3. **UI Layer** (`ui.rs`): Immediate mode GUI using egui

See [CLAUDE.md](CLAUDE.md) for detailed architecture documentation.

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check for errors without building
cargo check
```

### Database

The application uses SQLite with Diesel ORM. The database file (`bills.db`) is created automatically on first run. Migrations are embedded and run automatically.

To work with migrations:

```bash
# Install diesel CLI
cargo install diesel_cli --no-default-features --features sqlite

# Create a new migration
diesel migration generate migration_name

# Run migrations manually (usually automatic)
diesel migration run
```

## Technologies

- **[egui](https://github.com/emilk/egui)**: Immediate mode GUI framework
- **[Diesel](https://diesel.rs/)**: ORM and query builder for SQLite
- **[Typst](https://typst.app/)**: Modern document formatting for PDF generation
- **[qrcode](https://crates.io/crates/qrcode)**: QR code generation
- **[iso_11649](https://crates.io/crates/iso_11649)**: SCOR reference number generation
- **[iban](https://crates.io/crates/iban)**: IBAN validation

## Data Storage

- **Database**: `bills.db` in the application directory
- **Typst Packages**: `~/.cache/typst/packages/` (platform-specific)
- **Templates**: Expected in `templates/` directory

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]
