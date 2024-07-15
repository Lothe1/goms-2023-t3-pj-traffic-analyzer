use ta::kafka;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    kafka::consumer::start_enricher_to_tsdb().await;
    Ok(())
}