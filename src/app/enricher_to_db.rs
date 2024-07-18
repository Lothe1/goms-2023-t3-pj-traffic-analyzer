use ta::kafka;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    for _ in 0..10{
        tokio::spawn(async move {
            kafka::consumer::start_listener_to_enricher().await;
        });
    }
    Ok(())
}