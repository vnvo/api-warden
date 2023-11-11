use super::schema_tracker::SchemaTracker;
use crate::processor::schema_tracker;
use schema_analysis::targets::json_typegen::{Options, OutputMode, Shape};
use schema_analysis::InferredSchema;
use serde::de::DeserializeSeed;
use serde::Deserialize;
use serde_json::value::RawValue;
use std::error::Error;
use std::result::Result;
use std::str::Split;
use url;

/*
    recieve messages
    build and track schema per "key"
        * key can be the service
    update schema with each sample
    time based commits
        commiting means saving the last version and starting again
    check


*/
#[derive(Deserialize, Debug)]
//#[serde(from = "InferredSchema")]
pub struct ReqSchema(InferredSchema);
/* impl From<InferredSchema> for ReqSchema {
    type Error = String;

    fn from(value: InferredSchema) -> Self {

    }
} */

#[derive(Deserialize, Debug)]
pub struct RESTTransaction<'a> {
    pub source: String, //app-name-1
    pub method: String, // get,  post, ...
    pub uri: String,    //http://blah.com/a/path
    //pub req_params: Value, //param1=value1 {"param1":"value1"}
    #[serde(borrow)]
    pub req_hdr: &'a RawValue,
    #[serde(borrow)]
    pub req: &'a RawValue,
    #[serde(borrow)]
    pub resp_hdr: &'a RawValue,
    #[serde(borrow)]
    pub resp: &'a RawValue,
    #[serde(borrow)]
    pub ts: &'a RawValue,
}

impl<'a> RESTTransaction<'a> {
    pub fn get_key(&self) -> String {
        format!(
            "{}-{}-{}",
            self.source,
            self.method,
            mask_uri(&self.uri, Some(""))
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct TNXParseError;

pub struct Processor {
    tracker: SchemaTracker,
}

impl Processor {
    pub fn new() -> Processor {
        let st = schema_tracker::SchemaTracker::new();
        Processor { tracker: st }
    }

    pub fn process_transaction<'a>(&'a mut self, tnx: &'a str) -> Result<(), TNXParseError> {
        //let shallow_parsed: RESTTransaction = serde_json::from_str(tnx).unwrap();
        let shallow_res: Result<RESTTransaction, TNXParseError> = match serde_json::from_str(tnx) {
            Ok(v) => Ok(v),
            Err(error) => {
                eprintln!("{}", error);
                Err(TNXParseError {})
            }
        };

        let tnx_shallow = shallow_res.unwrap();
        let tnx_key = tnx_shallow.get_key();
        self.tracker.update(tnx_key.clone(), &tnx_shallow);

        Ok(())
    }
}

#[inline(always)]
pub fn mask_uri(uri: &str, server: Option<&str>) -> String {
    let uri_parsed = url::Url::parse(uri).unwrap();
    let host: String = match server {
        Some(s) => s.to_owned(),
        None => uri_parsed.host().unwrap().to_string(),
    };

    format!(
        "{}://{}/{}",
        uri_parsed.scheme(),
        host,
        mask_uri_path(uri_parsed.path_segments().unwrap())
    )

    //"".to_string()
}

#[inline(always)]
fn mask_uri_path<'a>(path: Split<'_, char>) -> String {
    let mut masked_path: Vec<&str> = vec![];

    for segment in path {
        if segment.chars().all(char::is_numeric) {
            masked_path.push("+d");
            continue;
        }

        masked_path.push(segment)
    }

    masked_path.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_uri() {
        let uri = "http://host-1/a/123123/c?p1=1&p2=test";
        let ret = mask_uri(uri, Some("test-host"));
        assert_eq!(ret, "http://test-host/a/+d/c");
        let ret = mask_uri(uri, None);
        assert_eq!(ret, "http://host-1/a/+d/c");

        println!("{:?}", ret);
    }

    #[test]
    fn test_process_transaction() {
        let tnx1: &str = r#"{
            "source":"app-1", 
            "uri":"http://test.app.com", 
            "req_hdr":{"a":"b"},
            "method":"get",
            "req":{},
            "resp_hdr":{"c":"d"},
            "resp":{"page":2, "per_page":10, "x":1, "data":[{"id":1, "name":"test", "email":""},{"id":2, "name":"test2", "email":"user@domain.com"}]},
            "ts":1273242342423
        }"#;

        let tnx2: &str = r#"{
            "source":"app-1", 
            "uri":"http://test.app.com", 
            "req_hdr":{"a":"b"},
            "method":"get",
            "req":{},
            "resp_hdr":{"c":"d"},
            "resp":{"page":20, "per_page":5, "x":"b", "data":[{"id":37, "name":"the name", "email":"name@mydomain.ee"},{"id":50, "name":"5hjrrr"}]},
            "ts":12316663424235
        }"#;

        let tnx3: &str = r#"{
            "source":"app-1", 
            "uri":"http://test.app.com", 
            "req_hdr":{"a":"b", "another-header":"and-its-value"},
            "method":"get",
            "req":{},
            "resp_hdr":{"c":"d", "header-2":"thy value"},
            "resp":{"page":4, "per_page":12, "x":[], "data":[{"id":100, "name":"sdffsdfsdh"},{"id":65, "name":"35613444-hhh"}]},
            "ts":12999884423423
        }"#;

        //setup and seeding with the first sample
        let mut st = schema_tracker::SchemaTracker::new();
        let tnx: RESTTransaction = serde_json::from_str(tnx1).unwrap();
        let key = tnx.get_key();

        // update with additional samples
        for raw in vec![tnx2, tnx3] {
            let tnx: RESTTransaction = serde_json::from_str(raw).unwrap();
            let key = tnx.get_key();
            st.update(key, &tnx);
        }

        // check the latest inferred schema with different outputs
        let v = st.get(key.as_str()).expect("");
        println!("=++++++\n{:#?}\n=+++++", v);

        for output_mode in vec![OutputMode::JsonSchema, OutputMode::Typescript] {
            let output: String = v
                .resp_schema
                .as_ref()
                .unwrap()
                .schema
                .process_with_json_typegen(output_mode.clone())
                .unwrap();

            println!("\n===== {:?} =====\n{}\n=====", output_mode, output);
        }
    }
}
