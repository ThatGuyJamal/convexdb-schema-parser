use convex_typegen::{generate, Configuration};

fn main()
{
    // Rebuild if the schema or games files change
    println!("cargo:rerun-if-changed=convex/schema.ts");
    println!("cargo:rerun-if-changed=convex/games.ts");

    let config = Configuration::default();

    // Generate the types
    match generate(config) {
        Ok(_) => {}
        Err(e) => panic!("Typegen failed: {}", e),
    };
}
