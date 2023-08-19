use super::process::RESTTransaction;
use schema_analysis::InferredSchema;
use serde::de::{DeserializeSeed, Deserializer};
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Clone)]
pub struct SchemaContainer {
    pub req_hdr_schema: Option<InferredSchema>,
    pub req_schema: Option<InferredSchema>,
    pub resp_hdr_schema: Option<InferredSchema>,
    pub resp_schema: Option<InferredSchema>,
    pub uri: String,
    pub source: String,
    samples: u32,
}

impl SchemaContainer {
    fn new(tnx: &RESTTransaction) -> SchemaContainer {
        SchemaContainer {
            req_hdr_schema: Some(serde_json::from_str(tnx.req_hdr.get()).unwrap()),
            req_schema: Some(serde_json::from_str(tnx.req.get()).unwrap()),
            resp_hdr_schema: Some(serde_json::from_str(tnx.resp_hdr.get()).unwrap()),
            resp_schema: Some(serde_json::from_str(tnx.resp.get()).unwrap()),
            uri: "".to_owned(),
            source: "".to_owned(),
            samples: 1,
        }
    }

    fn update_from(&mut self, tnx: &RESTTransaction) {
        let mut json_des = serde_json::Deserializer::from_str(tnx.req_hdr.get());
        self.req_hdr_schema = Option::<InferredSchema>::deserialize(&mut json_des).unwrap();

        let mut json_des = serde_json::Deserializer::from_str(tnx.req.get());
        self.req_schema = Option::<InferredSchema>::deserialize(&mut json_des).unwrap();

        let mut json_des = serde_json::Deserializer::from_str(tnx.resp_hdr.get());
        self.resp_hdr_schema = Option::<InferredSchema>::deserialize(&mut json_des).unwrap();

        let mut json_des = serde_json::Deserializer::from_str(tnx.resp.get());
        self.resp_schema = Option::<InferredSchema>::deserialize(&mut json_des).unwrap();

        self.samples += 1;
    }
}

pub struct SchemaTracker {
    schemas: HashMap<String, SchemaContainer>,
    commit_after: u32,
}

impl SchemaTracker {
    pub fn new() -> SchemaTracker {
        let hm = HashMap::<String, SchemaContainer>::new();
        SchemaTracker { 
            schemas: hm,
            commit_after: 5, 
        }
    }

    pub fn update(&mut self, key: String, tnx: &RESTTransaction) {
        let e = self.schemas.entry(key);

        match e {
            Entry::Occupied(mut entry) => {
                let sc = entry.get_mut();
                sc.update_from(tnx);

                if sc.samples > self.commit_after {
                    self.commit_schema();
                }
            },
            Entry::Vacant(vacant) => {
                vacant.insert(SchemaContainer::new(tnx));
            },
        }
    }

    pub fn get(&self, key: &str) -> Option<&SchemaContainer> {
        if self.schemas.contains_key(key) {
            let val = self.schemas.get(key).clone().unwrap();
            return Some(val);
        }

        None
    }

    fn commit_schema(&self) {}
}
