fn main() {
    // Configure the build to discover template files
    tomplate_build::Builder::new()
        .add_pattern("**/*.tomplate.toml")
        .add_pattern("templates/*.toml")
        .build()
        .expect("Failed to build templates");
}