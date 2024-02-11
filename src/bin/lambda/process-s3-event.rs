use std::{env, io::Cursor, process::exit};

use aws_config::{load_defaults, BehaviorVersion};
use aws_lambda_events::{event::s3::S3Event, serde_json};
use aws_sdk_s3::Client as S3Client;
use aws_sdk_sqs::{types::SendMessageBatchRequestEntry, Client as SqsClient};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use sam_rust_inventory::model::Record;
use sam_rust_inventory::s3::GetFile;

use tracing::{error, info};

/// Main function
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // Initialize the AWS SDK for Rust
    let config = load_defaults(BehaviorVersion::v2023_11_09()).await;

    let sqs_client = SqsClient::new(&config);
    let s3_client = S3Client::new(&config);
    let bucket_name = env::var("BUCKET").expect("Couldn't get the bucket");
    let queue_name = env::var("QUEUE").expect("Couldn't get the queue");
    let bucket_name_ref = &bucket_name;
    let s3_client_ref = &s3_client;
    let sqs_client_ref = &sqs_client;
    let queue_name_ref = &queue_name;

    let func = service_fn(move |event| async move {
        function_handler(
            event,
            bucket_name_ref,
            s3_client_ref,
            sqs_client_ref,
            queue_name_ref,
        )
        .await
    });

    run(func).await?;
    Ok(())
}

async fn function_handler<T: GetFile>(
    event: LambdaEvent<S3Event>,
    bucket_name: &String,
    client: &T,
    sqs_client: &SqsClient,
    sqs_queue_url: &String,
) -> Result<(), Error> {
    let mut message_batch: Vec<SendMessageBatchRequestEntry> = Vec::new();

    let records = event.payload.records;
    info!("Bucket name is {}", bucket_name);
    let key = match &records[0].s3.object.key {
        Some(key) => key,
        None => {
            error!("Couldn't retrieve key");
            println!("Couldn't retrieve key");
            return Err("Unable to retrieve item key".into());
        }
    };

    info!("bucket and key are {} and {}", bucket_name, key);

    let csv_file = match client.get_file(&bucket_name, &key).await {
        Ok(vec) => vec,
        Err(err) => {
            println!("Can not get file from S3: {}", err);
            info!("Can not get file from S3: {}", err);
            exit(1);
        }
    };

    info!("csv file is {:?}", csv_file);

    let reader = Cursor::new(csv_file);

    let mut rdr = csv::Reader::from_reader(reader);
    for result in rdr.deserialize() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here..
        let record: Record = result?;
        info!(" records are {:?}", record);

        let sqs_message_body = SendMessageBatchRequestEntry::builder()
            .id((message_batch.len() + 1).to_string())
            .message_body(serde_json::to_string(&record).unwrap())
            .build()?;

        message_batch.push(sqs_message_body);

        if message_batch.len() == 10 {
            let rsp = sqs_client
                .send_message_batch()
                .queue_url(sqs_queue_url)
                .set_entries(Some(message_batch.clone()))
                .send()
                .await;

            message_batch = vec![];

            info!("Send message to the queue: {:#?}", rsp);
        }
    }

    if message_batch.len() != 0 {
        let rsp = sqs_client
            .send_message_batch()
            .queue_url(sqs_queue_url)
            .set_entries(Some(message_batch.clone()))
            .send()
            .await;

        info!("Send message to the queue: {:#?}", rsp);
    }
    Ok(())
}
