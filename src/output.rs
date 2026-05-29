use crate::product::Product;

/// Render a list of search results.
pub fn print_search(products: &[Product], json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(products).expect("serialize products")
        );
    } else {
        for (i, p) in products.iter().enumerate() {
            print_human(p, i + 1);
        }
    }
}

/// Render a single product.
pub fn print_one(p: &Product, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(p).expect("serialize product")
        );
    } else {
        print_human(p, 0);
    }
}

fn print_human(p: &Product, idx: usize) {
    let num = if idx > 0 {
        format!("{idx}. ")
    } else {
        String::new()
    };
    let price = p.price.as_deref().unwrap_or("—");
    println!("{num}{}  [{price}]", p.title);
    if let Some(brand) = p.brand.as_deref() {
        if !brand.is_empty() {
            println!("   brand: {brand}");
        }
    }
    println!("   tcin:  {}", p.tcin);
    if let Some(u) = &p.url {
        println!("   url:   {u}");
    }
    println!();
}
