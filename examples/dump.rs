//! Prints the merged, categorised cheatsheet to stdout.
fn main() {
    cosmic_ext_cheatsheet::i18n::init_from_desktop();
    let model = cosmic_ext_cheatsheet::shortcuts::CheatsheetModel::load();
    let query: String = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let model = model.filter(&query);
    for category in &model.categories {
        println!("\n== {} ({})", category.title, category.entries.len());
        for entry in &category.entries {
            let combos: Vec<String> = entry.bindings.iter().map(|b| b.join("+")).collect();
            println!("  {:<45} {}", entry.label, combos.join("   |   "));
        }
    }
    println!("\ntotal entries: {}", model.len());
    println!("spawn command: {}", cosmic_ext_cheatsheet::registration::spawn_command());
}
