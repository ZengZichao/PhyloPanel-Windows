use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: dump_schema <binary> [schema.json]");
        exit(2);
    }
    let schema = match phylo_panel::schema::Reflector::new(&args[0]).reflect() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {}", e);
            exit(1);
        }
    };

    let mut groups: Vec<String> = schema
        .commands
        .iter()
        .map(|c| c.group.clone())
        .collect();
    groups.sort();
    groups.dedup();
    let own: usize = schema
        .commands
        .iter()
        .map(|c| c.flags.iter().filter(|f| !f.inherited).count())
        .sum();
    let nested = schema.commands.iter().filter(|c| c.path.len() > 1).count();
    let dual: Vec<&str> = schema
        .commands
        .iter()
        .filter(|c| c.path.len() > 1 && schema.commands.iter().any(|o| o.id == c.group))
        .map(|c| c.group.as_str())
        .collect();

    println!("version        : {}", schema.version);
    println!("leaf commands  : {} ({} nested)", schema.commands.len(), nested);
    println!("top-level sets : {}", groups.len());
    println!("own flags      : {} (avg {:.1})", own, own as f64 / schema.commands.len().max(1) as f64);
    println!("parent that is also runnable: {:?}", {
        let mut d: Vec<&str> = dual.into_iter().collect();
        d.sort();
        d.dedup();
        d
    });

    for id in ["draw svg", "acr", "cut", "reroot outgroup"] {
        let Some(c) = schema.commands.iter().find(|c| c.id == id) else { continue };
        println!("\n--- {} : {}", id, c.description.replace('\n', " / "));
        for f in &c.flags {
            println!(
                "    --{:.<22}{:?}{:>10} def={:<12} {:?}{}",
                f.name,
                f.short,
                format!("{:?}", f.kind),
                f.default.clone().unwrap_or_default(),
                f.role,
                f.choices
                    .as_ref()
                    .map(|c| format!("  choices={}", c.join("|")))
                    .unwrap_or_default()
            );
        }
    }

    if let Some(path) = args.get(1) {
        std::fs::write(path, serde_json::to_string_pretty(&schema).unwrap()).expect("write");
        println!("\nwrote {}", path);
    }
}
