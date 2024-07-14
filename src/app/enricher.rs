#![allow(unused_imports)]




use ta::app::consumer::start_listener_to_enricher;



#[tokio::main]
async fn main() -> std::io::Result<()> {
    start_listener_to_enricher().await;
    Ok(())
}
