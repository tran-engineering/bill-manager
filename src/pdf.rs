use std::collections::HashMap;
use std::fmt::format;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::app::{Bill, Client};
use crate::types::Address;

use text_placeholder::Template;
use typst::diag::{FileError, FileResult, PackageError, PackageResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath, package::PackageSpec};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::fonts::{FontSearcher, FontSlot};
use typst_pdf::PdfOptions;

static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(|| {
    LazyHash::new(Library::builder().build())
});


struct TypstWorld {
    source: Source,
    main_id: FileId,
    package_cache: PathBuf,
    template_dir: PathBuf,
    book: LazyHash<FontBook>,
    fonts: Vec<FontSlot>,
}

impl TypstWorld {
    fn new(source_text: String) -> Self {
        let main_id = FileId::new(None, VirtualPath::new("main.typ"));
        let source = Source::new(main_id, source_text);

        // Use system cache directory for packages
        let package_cache = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("typst")
            .join("packages");

        // Template directory for local files
        let template_dir = PathBuf::from("templates");

        let fonts = FontSearcher::new().include_system_fonts(true).search();
        let book = LazyHash::new(fonts.book);

        Self {
            source,
            main_id,
            package_cache,
            template_dir,
            fonts: fonts.fonts,
            book,
        }
    }

    fn resolve_package(&self, spec: &PackageSpec) -> PackageResult<PathBuf> {
        let package_dir = self.package_cache
            .join(spec.namespace.as_str())
            .join(spec.name.as_str())
            .join(spec.version.to_string());

        if package_dir.exists() {
            Ok(package_dir)
        } else {
            // Try to download the package
            self.download_package(spec)
        }
    }

    fn download_package(&self, spec: &PackageSpec) -> PackageResult<PathBuf> {
        let package_dir = self.package_cache
            .join(spec.namespace.as_str())
            .join(spec.name.as_str())
            .join(spec.version.to_string());

        if package_dir.exists() {
            return Ok(package_dir);
        }

        // Create the directory
        fs::create_dir_all(&package_dir)
            .map_err(|e| PackageError::Other(Some(format!("Failed to create package directory: {}", e).into())))?;

        // Download from Typst package registry
        let url = format!(
            "https://packages.typst.org/{}/{}-{}.tar.gz",
            spec.namespace,
            spec.name,
            spec.version
        );

        // For now, return an error with instructions
        Err(PackageError::Other(Some(
            format!(
                "Package '{}' not found in cache. Please download it manually:\n\
                 1. Download from: {}\n\
                 2. Extract to: {}\n\
                 Or run: typst compile (with the CLI) to auto-download packages",
                spec,
                url,
                package_dir.display()
            ).into()
        )))
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main_id {
            Ok(self.source.clone())
        } else if let Some(package_spec) = id.package() {
            // Handle package files
            let package_dir = self.resolve_package(package_spec)
                .map_err(|e| FileError::Package(e))?;

            let file_path = package_dir.join(id.vpath().as_rootless_path());

            let text = fs::read_to_string(&file_path)
                .map_err(|_| FileError::NotFound(file_path))?;

            Ok(Source::new(id, text))
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if let Some(package_spec) = id.package() {
            // Handle package files
            let package_dir = self.resolve_package(package_spec)
                .map_err(|e| FileError::Package(e))?;

            let file_path = package_dir.join(id.vpath().as_rootless_path());

            let data = fs::read(&file_path)
                .map_err(|_| FileError::NotFound(file_path))?;

            Ok(Bytes::new(data))
        } else {
            // Handle local files relative to template directory
            let file_path = self.template_dir.join(id.vpath().as_rootless_path());

            let data = fs::read(&file_path)
                .map_err(|_| FileError::NotFound(file_path))?;

            Ok(Bytes::new(data))
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts[index].get()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Datetime::from_ymd(2024, 1, 1)
    }
}

pub fn generate_bill_pdf(
    bill: &Bill,
    client: &Client,
    creditor: &Address,
) -> Result<Vec<u8>, String> {
    let typst_content = create_typst_invoice(bill, client, creditor);

    let world = TypstWorld::new(typst_content);

    let result = typst::compile(&world);
    let document = result.output
        .map_err(|errors| format!("Typst compilation failed: {:?}", errors))?;

    let pdf_data = typst_pdf::pdf(&document, &PdfOptions::default())
        .map_err(|e| format!("PDF generation failed: {:?}", e))?;

    Ok(pdf_data)
}

fn create_typst_invoice(bill: &Bill, client: &Client, creditor: &Address) -> String {
    let template_str = fs::read_to_string("templates/qr_bill.tpl").unwrap();

    let tpl = Template::new(&template_str);

    let amount_str = bill.total().to_string();

    let table_contents = "[testbesch], [softw], [125.00]".to_string();

    let bla = bill.items.iter().fold(String::new(), |all, item| all + format!("[{}], [{}], [{}], [{}], [{}]", item.note, item.description, item.quantity, item.unit_price, item.total()).to_string());

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
        ("debtor-name", client.billing_address.name.as_str()),
        ("debtor-street", client.billing_address.street.as_deref().unwrap_or("")),
        ("debtor-building", client.billing_address.building_number.as_deref().unwrap_or("")),
        ("debtor-postal-code", client.billing_address.postal_code.as_str()),
        ("debtor-city", client.billing_address.city.as_str()),
        ("debtor-country", client.billing_address.country.as_str()),
        ("reference-type", "SCOR"),
        ("reference", bill.reference.as_str()),
        ("additional-info", bill.notes.as_str()),
        ("table-contents", table_contents.as_str())
    ]);

    tpl.fill_with_hashmap(&vars)
}
