use rdkafka::consumer::{Consumer, StreamConsumer, CommitMode};

use rdkafka::{ClientConfig, Message};
use rdkafka::producer::FutureProducer;

pub async fn start(){
    let consumer: StreamConsumer = create();
    consume(consumer).await;
}

fn create() ->StreamConsumer {
    let mut config = ClientConfig::new();

    config.set("bootstrap.servers", "localhost:9092");
    config.set("auto.offset.reset", "earliest");
    config.set("group.id", "test-group");
    config.set("socket.timeout.ms", "4000");
    let consumer: StreamConsumer =
        config.create()
            .expect("Consumer creation failed");

    consumer
}

async fn consume(consumer:StreamConsumer){
    consumer.subscribe
        (
            &["test-topic"]
        )
        .expect("Can't subscribe to specified topic");

    loop{
        match consumer.recv().await{
            Err(e) => println!("Error receiving message: {:?}", e),
            Ok(message) => {
                match message.payload_view::<str>(){
                    None => println!("NO message"),
                    Some(Ok(s)) => {
                        println!("Message: {:?}", s);
                    },
                    Some(Err(e)) => {
                        println!("Error unpacking message: {:?}", e);
                    }
                }
                consumer.commit_message(&message, CommitMode::Async).unwrap();


            }
        }
    }
}

