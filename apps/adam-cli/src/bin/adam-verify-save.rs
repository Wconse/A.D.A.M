use adam_content::save_file::read_save_file;
use std::path::Path;
fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: adam-verify-save <file>");
        std::process::exit(2);
    };
    match read_save_file(Path::new(&path)) {
        Ok(payload) => println!("valid save container: payload_bytes={}", payload.len()),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
