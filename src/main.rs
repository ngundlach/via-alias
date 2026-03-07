#[tokio::main]
async fn main() {
    if let Err(e) = via_alias::run_app().await {
        eprintln!("Via-Alias encountered an error: {e}");
    }
}
