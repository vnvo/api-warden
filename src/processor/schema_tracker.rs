use std::collections::HashMap;
use schema_analysis::InferredSchema;
use serde::Deserialize;
use serde::de::{DeserializeSeed, Deserializer, IntoDeserializer};
use super::process::RESTTransaction;

#[derive(Debug, Clone)]
pub struct SchemaContainer {
    pub req_hdr_schema: Option<InferredSchema>,
    pub req_schema: Option<InferredSchema>,
    pub resp_hdr_schema: Option<InferredSchema>,
    pub resp_schema: Option<InferredSchema>,
    pub uri: String,
    pub source: String
}

#[derive(Debug)]
pub enum SchemaTrackerErr {
    FailedToStart(String),
    KeyNotFound
}

pub struct SchemaTracker {
    schemas: HashMap<String, SchemaContainer>
}


impl SchemaTracker {

    pub fn new() -> SchemaTracker {
        let hm = HashMap::<String, SchemaContainer>::new();
        SchemaTracker {
            schemas: hm
        }
    }
    
    pub fn update (&mut self, key: String, tnx: &RESTTransaction) -> Result<(), SchemaTrackerErr> {

        self.schemas
            .entry(key)
            .and_modify(|sch| {
                //let v = tnx.resp.to_string();
                let mut json_deserializer = serde_json::Deserializer::from_str(tnx.resp.get());
                sch.resp_schema = Option::<InferredSchema>::deserialize(&mut json_deserializer).unwrap();

            })
            .or_insert_with(|| {
                let resp_inferred: InferredSchema = serde_json::from_str(tnx.resp.get()).unwrap();
                SchemaContainer{
                    req_hdr_schema: None, 
                    req_schema: None, 
                    resp_hdr_schema: None, 
                    resp_schema: Some(resp_inferred), 
                    uri: "".to_owned(), 
                    source: "".to_owned() 
                }
            }
        );

        //Err(SchemaTrackerErr::FailedToStart("".to_string()))
        Ok(())
    }
    
     pub fn get (&self, key: &str) -> Option<&SchemaContainer> {
        if self.schemas.contains_key(key) {
            let val = self.schemas.get(key).clone().unwrap();
            return Some(val);
        }

        None
    }

}
