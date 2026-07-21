use adam_content::mod_validation::validate_mod_folder;
use std::path::Path;
fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "mods/example".into());
    match validate_mod_folder(Path::new(&path)) {
        Ok(report) => println!(
            "valid mod: {} {} goods={} recipes={}",
            report.manifest.id().as_str(),
            report.manifest.version(),
            report.goods,
            report.recipes
        ),
        Err(issues) => {
            for issue in issues {
                eprintln!("{issue}");
            }
            std::process::exit(2);
        }
    }
}
