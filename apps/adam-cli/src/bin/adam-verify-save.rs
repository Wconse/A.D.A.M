use adam_content::save_file::{SaveSource, read_save_with_backup};
use std::path::Path;
fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: adam-verify-save <file>");
        std::process::exit(2);
    };
    match read_save_with_backup(Path::new(&path)) {
        Ok(result) => {
            let source = match result.source {
                SaveSource::Primary => "primary",
                SaveSource::Backup => "backup",
            };
            println!(
                "valid save container: source={source} payload_bytes={}",
                result.payload.len()
            );
            if let Some(error) = result.primary_error {
                eprintln!("primary invalid: {error}");
            }
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            std::process::exit(2);
        }
    }
}
