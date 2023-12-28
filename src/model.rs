use serde::{Deserialize,Serialize};

// By default, struct field names are deserialized based on the position of
// a corresponding field in the CSV data's header record.
#[derive(Debug, Deserialize,Serialize)]
pub struct Record {
    pub product_id:String,
    pub location:String,
    pub quantity:u32,
    pub update_date:String
  
}
