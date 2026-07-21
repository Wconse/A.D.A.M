use adam_content::mod_validation::validate_mod_set;
use std::path::PathBuf;
fn main() {
    let mut paths: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    if paths.is_empty() {
        paths.push("mods/example".into());
    }
    match validate_mod_set(&paths) {
        Ok(report) => {
            println!(
                "valid mod set: goods={} recipes={} fingerprint={:016x}",
                report.goods, report.recipes, report.package_fingerprint
            );
            for id in report.load_order {
                println!("load: {}", id.as_str());
            }
        }
        Err(issues) => {
            for issue in issues {
                eprintln!("{issue}");
            }
            std::process::exit(2);
        }
    }
}
