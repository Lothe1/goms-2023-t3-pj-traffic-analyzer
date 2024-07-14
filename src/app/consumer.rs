use rdkafka::consumer::{Consumer, StreamConsumer, CommitMode};
use rdkafka::{ClientConfig, Message};

use crate::influx_db::CustomMessage;


pub async fn start_listener_to_enricher(){
    let consumer: StreamConsumer = create();
    consume_listener_to_enricher(consumer).await;
}

pub async fn start_enricher_to_tsdb(){
    let consumer: StreamConsumer = create();
    consume_enricher_to_tsdb(consumer).await;
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

async fn consume_listener_to_enricher(consumer:StreamConsumer){
    consumer.subscribe
        (
            &["listener-to-enricher"]
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

async fn consume_enricher_to_tsdb(consumer:StreamConsumer){
    consumer.subscribe
    (
        &["enricher-to-tsdb"]
    )
        .expect("Can't subscribe to specified topic");

    loop{
        match consumer.recv().await{
            Err(e) => println!("Error receiving message: {:?}", e),
            Ok(message) => {
                match message.payload_view::<str>(){
                    None => println!("NO message"),
                    Some(Ok(s)) => {
                        let recieved_message: CustomMessage = serde_json::from_str(s).expect("Failed to deserialize message");
                        println!("Message: {:?}", recieved_message);
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



