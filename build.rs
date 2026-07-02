fn main() {
    include_packed::Config::new("assets")
        .level(20)
        .build()
        .expect("Failed to pack assets");
}
