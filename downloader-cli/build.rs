extern crate embed_resource;

fn main() {
    if std::env::var("CARGO_FEATURE_PRODUCTION").is_ok() {
        embed_resource::compile("app.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap_or_else(|e| {
                eprintln!("Failed to compile resources: {e}");
            });
    }
}
