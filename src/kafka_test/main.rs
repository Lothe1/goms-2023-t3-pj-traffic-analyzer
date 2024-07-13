use crate::producer::produce;

mod producer;
mod consumer;


#[tokio::main]
async fn main() {
    let producer = producer::create();
    produce(&producer, "Hello from Rust").await;
    produce(&producer, "Hello again").await;
    produce(&producer, "Hello once more").await;
    consumer::start().await;

}

