use rdkafka::{
    ClientConfig, Message,
    client::ClientContext,
    consumer::{Consumer, StreamConsumer},
    producer::FutureProducer,
};
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    errors::KafkaConnectionError,
    queue_messages::{ImageFeed, MinioKakfaEvent},
};

#[derive(Error, Debug)]
pub enum MessagingError {
    #[error("Failed to create producer")]
    ProducerCreate,
    #[error("Failed to create consumer")]
    ConsumerCreate,
}

fn create_producer(server_path: &str) -> Result<FutureProducer, MessagingError> {
    Ok(ClientConfig::new()
        .set("bootstrap.servers", server_path)
        .set("queue.buffering.max.ms", "0")
        .create()
        .map_err(|op| {
            // Should probably just break the program
            log::error!("{op:?}");
            MessagingError::ProducerCreate
        })?)
}

pub fn create_consumer(server_path: &str) -> Result<StreamConsumer, MessagingError> {
    Ok(ClientConfig::new()
        .set("bootstrap.servers", server_path)
        .set("enable.partition.eof", "false")
        .set("group.id", format!("imgfeeder-{}", Uuid::new_v4()))
        .set_log_level(rdkafka::config::RDKafkaLogLevel::Debug)
        .create()
        .map_err(|op| {
            println!("error error error");
            println!("{op:?}");
            MessagingError::ConsumerCreate
        })?)
}

pub async fn feeder_protocol(
    consumer: StreamConsumer,
    topics: Vec<&str>,
    feed_producer: mpsc::UnboundedSender<ImageFeed>,
) -> Result<(), KafkaConnectionError> {
    consumer.subscribe(&topics).map_err(|err| {
        log::error!("{err:?}");
        KafkaConnectionError::Subscribe
    })?;

    loop {
        let message = consumer
            .recv()
            .await
            .map_err(|err| {
                log::error!("{err:?}");
                KafkaConnectionError::RecvMessage
            })?
            .detach();

        if let Some(key) = message.key() {
            log::debug!("Received message with key {:#?}", std::str::from_utf8(key));
            if key.starts_with(b"rag-upload/rag-thumbnail") {
                continue;
            }
        } else {
            log::warn!("Received message without key");
            continue;
        }

        let payload = match message.payload() {
            Some(p) => p,
            None => {
                log::warn!("Received message without payload");
                continue;
            }
        };

        let payload = serde_json::from_slice::<MinioKakfaEvent>(payload).map_err(|err| {
            log::error!("{err:?}");
            KafkaConnectionError::RecvMessage
        })?;
        for record in &payload.records {
            feed_producer.send(record.into()).map_err(|err| {
                log::error!("{err:?}");
                KafkaConnectionError::MpscSendMessage
            })?;
        }
    }
}

#[cfg(test)]
mod tests {
    use rdkafka::{
        Message, Offset, TopicPartitionList,
        consumer::{CommitMode, Consumer},
        producer::FutureRecord,
        util::Timeout,
    };

    use crate::queue::{create_consumer, create_producer};

    #[tokio::test()]
    async fn test_send_receive() {
        let server_path = "localhost:9092";
        let topic = "test_feeder";
        let message_key = "key_message";
        let message_payload = "This is a test message";

        let consumer = create_consumer(server_path).expect("Failed to create consumer.");
        let producer = create_producer(server_path).expect("Failed to create producer.");

        // let mut topic_partitions = TopicPartitionList::new();
        // topic_partitions
        //     .add_partition_offset("chat", 0, Offset::Offset(0))
        //     .unwrap();
        // consumer
        //     .commit(&topic_partitions, CommitMode::Async)
        //     .unwrap();
        consumer.subscribe(&[topic]).expect("Failed to subscribe.");

        // Setup timeout
        let mut ticket = tokio::time::interval(std::time::Duration::from_millis(1500));
        ticket.tick().await; // This should be immediate

        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            let number = 10;
            for num in 0..number {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                producer
                    .send(
                        FutureRecord::to(topic)
                            .key(&format!("message_key_{num}"))
                            .payload(message_payload.as_bytes()),
                        Timeout::Never,
                    )
                    .await
                    .expect("Failed to send message.");
            }
        });

        loop {
            tokio::select! {
               incomming_msg =  consumer.recv() => {
                    let incomming_msg = incomming_msg.expect("Failed to read message").detach();
                    let key = incomming_msg.key().expect("No key on message");

                    assert!(key != message_key.as_bytes(), "{:#?} {}", std::str::from_utf8(key), message_key );

                    let payload = incomming_msg.payload().expect("No payload on message");
                    assert!(payload == message_payload.as_bytes());

                    break;
                }
                 _tick = ticket.tick() => {
                     assert!(false, "Timeout");
                 }
            }
        }
    }
}
