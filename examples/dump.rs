//! Prints the merged, categorised cheatsheet to stdout.
fn main() {
    cosmic_ext_cheatsheet::i18n::init_from_desktop();
    let model = cosmic_ext_cheatsheet::shortcuts::CheatsheetModel::load();
    let query: String = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let search = model.search(&query);
    for matched in &search.categories {
        println!(
            "\n== {} ({})",
            matched.category.title,
            matched.entries.len()
        );
        for entry in &matched.entries {
            let combos: Vec<String> = entry.bindings.iter().map(|b| b.join("+")).collect();
            println!("  {:<45} {}", entry.label, combos.join("   |   "));
        }
    }
    println!("\ntotal entries: {}", model.len());
    if let Some(target) = search.enter_target() {
        println!(
            "enter runs: {} -> {}",
            target.label,
            target.command.as_deref().unwrap_or_default()
        );
    }
    println!(
        "spawn command: {}",
        cosmic_ext_cheatsheet::registration::spawn_command()
    );
}
