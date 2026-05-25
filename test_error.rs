use invo::config::parse_config_str;
use std::path::Path;

fn main() {
    let result = parse_config_str("identitty: {}\n", Path::new("test.yaml"));
    println!("Error: {:?}", result);
}