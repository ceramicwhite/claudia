fn main() {
    let with_hyphens = "550e8400-e29b-41d4-a716-446655440000";
    let without_hyphens = "550e8400e29b41d4a716446655440000";
    
    println\!("With hyphens: {:?}", uuid::Uuid::parse_str(with_hyphens));
    println\!("Without hyphens: {:?}", uuid::Uuid::parse_str(without_hyphens));
}
