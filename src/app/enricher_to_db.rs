#![allow(unused_imports)]


use ta::app::consumer::start_enricher_to_tsdb;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    start_enricher_to_tsdb().await;
    Ok(())
}