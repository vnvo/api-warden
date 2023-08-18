use super::process::RESTTransaction;
use schema_analysis::InferredSchema;
use serde::de::{DeserializeSeed, Deserializer};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SchemaContainer {
    pub req_hdr_schema: Option<InferredSchema>,
    pub req_schema: Option<InferredSchema>,
    pub resp_hdr_schema: Option<InferredSchema>,
    pub resp_schema: Option<InferredSchema>,
    pub uri: String,
    pub source: String,
}

impl SchemaContainer {
    fn update_from(&mut self, tnx: &RESTTransaction) {
        let mut json_des = serde_json::Deserializer::from_str(tnx.req_hdr.get());
        self.req_hdr_schema = Option::<InferredSchema>::deserialize(&mut json_des).unwrap();

        let mut json_des = serde_json::Deserializer::from_str(tnx.req.get());
        self.req_schema = Option::<InferredSchema>::deserialize(&mut json_des).unwrap();

        let mut json_des = serde_json::Deserializer::from_str(tnx.resp_hdr.get());
        self.resp_hdr_schema = Option::<InferredSchema>::deserialize(&mut json_des).unwrap();

        let mut json_des = serde_json::Deserializer::from_str(tnx.resp.get());
        self.resp_schema = Option::<InferredSchema>::deserialize(&mut json_des).unwrap();
    }
}

pub struct SchemaTracker {
    schemas: HashMap<String, SchemaContainer>,
}

impl SchemaTracker {
    pub fn new() -> SchemaTracker {
        let hm = HashMap::<String, SchemaContainer>::new();
        SchemaTracker { schemas: hm }
    }

    pub fn update(&mut self, key: String, tnx: &RESTTransaction) {
        self.schemas
            .entry(key)
            .and_modify(|sch| {
                sch.update_from(tnx);
            })
            .or_insert_with(|| {
                let resp_inferred: InferredSchema = serde_json::from_str(tnx.resp.get()).unwrap();
                SchemaContainer {
                    req_hdr_schema: None,
                    req_schema: None,
                    resp_hdr_schema: None,
                    resp_schema: Some(resp_inferred),
                    uri: "".to_owned(),
                    source: "".to_owned(),
                }
            });
    }

    pub fn get(&self, key: &str) -> Option<&SchemaContainer> {
        if self.schemas.contains_key(key) {
            let val = self.schemas.get(key).clone().unwrap();
            return Some(val);
        }

        None
    }
}
