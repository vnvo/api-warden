pub mod utils;
pub mod processor;

use processor::Processor;

use log::{info, warn};
use std::time::Duration;
use rdkafka::{
    ClientContext, 
    consumer::{ConsumerContext, StreamConsumer, Consumer, CommitMode}, 
    ClientConfig, 
    Offset,
    util::Timeout,
    config::RDKafkaLogLevel, Message, message::Headers
};

struct CustomContext;

impl ClientContext for CustomContext {}

impl  ConsumerContext for CustomContext {
    fn pre_rebalance<'a>(&self, rebalance: &rdkafka::consumer::Rebalance<'a>) {
        info!("pre balance: {:?}", rebalance);
    }

    fn post_rebalance<'a>(&self, rebalance: &rdkafka::consumer::Rebalance<'a>) {
        info!("post rebalance: {:?}", rebalance);
    }

    fn commit_callback(&self, result: rdkafka::error::KafkaResult<()>, offsets: &rdkafka::TopicPartitionList) {
        info!("committing offsets: {:?} - {:?}", offsets, result)
    }
}

type APIWardenConsumer = StreamConsumer<CustomContext>;

pub async fn consume_and_process(brokers: &str, group_id: &str, topics: &[&str]) {

    let mut proc = Processor::new();

    let context = CustomContext;
    let consumer: APIWardenConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("consumer creation failed");
    
    consumer
        .subscribe(&topics)
        .expect("can not subscribe to specific topics.");

    
    /* let topic_watermarks = consumer
        .fetch_watermarks(topics[0], 0, Timeout::After(Duration::from_millis(3000)))
        .expect("failed to get watermarks");
    println!("{:?}", topic_watermarks);
    */
    println!("{}", topics[0]);
    consumer.seek(topics[0], 0, 
        Offset::Offset(0), Timeout::After(Duration::from_millis(10000)))
        .expect("failed to seek");

    println!("{:#?}", consumer.position());

    loop {
        println!("consumer loop ...");
        match consumer.recv().await {
            Err(e) => warn!("kafka error: {}", e),
            Ok(m) => {
                println!("there is message ...");
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                if let Some(headers) = m.headers() {
                    for header in headers.iter() {
                        info!("    Header {:#?}: {:?}", header.key, header.value);
                    }
                }

                match proc.process_transaction(payload) {
                    Ok(_) => {},
                    Err(_) => eprintln!("error on processing payload"),
                }

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}
