


use std::env;

use aws_lambda_events::event::sqs::SqsEventObj;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use aws_config::{BehaviorVersion,load_defaults};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use sam_rust_inventory::model::Record;

use tracing::{info,error};


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

      let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let dynamodb_client = Client::new(&config);
     let table_name_ref = &table_name;
    let dynamodb_client_ref = &dynamodb_client;

   

        let func = service_fn(move |event| function_handler(event,table_name_ref, dynamodb_client_ref) );

    run(func).await?;
    Ok(())
  
}

async fn function_handler(
    event: LambdaEvent<SqsEventObj<Record>>,
   table_name:&String,
    client: &Client,
) -> Result<(), Error> {
  //info!("sqs payload {:?}",&event.payload);
  //info!("sqs payload records {:?}",&event.payload.records[0]);
  for data_more in &event.payload.records{

    let record_data = &data_more.body;
     info!("Data retrieved from sqs {:?}",data_more.body);

     let res =  client
        .put_item()
        .table_name(table_name)
         .item("id", AttributeValue::S(record_data.product_id.clone()))
         .item("location", AttributeValue::S(record_data.location.clone()))
         
           .item("quantity", AttributeValue::N(record_data.quantity.to_string()))
             .item("updated_on", AttributeValue::S(record_data.update_date.clone()))
       
        .send()
        .await;

      match res {
        Ok(output) =>{
            info!("Item successfully added to db {:?}",output)
        },
        Err(err) => {
            error!("An error occured while adding item to db {:?}",err);
            return Err(Box::new(err))
          
        },
    };
  }

 
    
    Ok(())
       
    }
