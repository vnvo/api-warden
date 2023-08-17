use tokio;
use api_warden::{consume_and_process, utils};
use utils::setup_logger;

#[tokio::main]
async fn main() {

    let brokers = "127.0.0.1:19092"; 
    let group_id = "test-api-warden";
    let topics = ["api-warden-aggr"];
    setup_logger(true, None);
    consume_and_process(brokers, group_id, &topics).await
}