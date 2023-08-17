use serde::Deserialize;
use serde::de::DeserializeSeed;
use serde_json::{Value, Map as SMap};
use std::result::Result;
use std::str::Split;
use schema_analysis::InferredSchema;
use schema_analysis::targets::json_typegen::{Shape, OutputMode, Options};
use crate::processor::schema_tracker;
use url;
use super::schema_tracker::SchemaTracker;

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
pub struct  RESTTransaction {
    pub source: String, //app-name-1
    pub method: String, // get,  post, ...
    pub uri: String, //http://blah.com/a/path
    //pub req_params: Value, //param1=value1 {"param1":"value1"}
    pub req_hdr: Value,
    pub req: Value,
    pub resp_hdr: Value,
    //#[serde(from = "InferredSchema")]
    pub resp: Value,
    pub ts: Value
}

impl RESTTransaction {
    fn get_key(&self) -> String {
        format!(
            "{}-{}-{}", 
            self.source,
            self.method, 
            mask_uri(&self.uri, Some(""))
        )
    }
}

#[derive(Deserialize, Debug)]
struct RESTParseError;

pub struct Processor {
    tracker: SchemaTracker
}

impl Processor {

    pub fn new() -> Processor {        
        let mut st = schema_tracker::SchemaTracker::new();
        Processor { tracker: st }
    }   

    pub fn process_transaction(&self, tnx: &str) -> Option<RESTTransaction> {
        //let shallow_parsed: RESTTransaction = serde_json::from_str(tnx).unwrap();
        let shallow_res: Result<RESTTransaction, RESTParseError> = match serde_json::from_str(tnx) {
            Ok(v) => Ok(v),
            Err(error) => {
                eprintln!("{}", error);
                Err(RESTParseError{})
            }
        };
        
        let shallow_res = shallow_res.unwrap();
        let tnx_key = shallow_res.get_key();
        // 
        let isch: InferredSchema = serde_json::from_str(tnx).unwrap();
        Some(shallow_res)
    }
    

    fn commit(&self, key: &str) {

    }
}


pub fn mask_uri(uri: &str, server: Option<&str>) -> String {
    let uri_parsed = url::Url::parse(uri).unwrap();
    let host: String = match server {
        Some(s) => s.to_owned(),
        None => uri_parsed.host().unwrap().to_string()
    };

    format!("{}://{}/{}", 
        uri_parsed.scheme(),
        host,
        mask_uri_path(uri_parsed.path_segments().unwrap())
    )

    //"".to_string()
}

fn mask_uri_path<'a>(path: Split<'_, char> ) -> String{
    let mut masked_path: Vec<&str> = vec!();

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
        
        let mut st = schema_tracker::SchemaTracker::new();
        let key = format!("{}_{}", "app-1", "http://test.app.com");
        let mut inferred: InferredSchema = serde_json::from_str(tnx1).unwrap();
        println!("{:?}", inferred.schema);
        //let sch = schemars::schema_for_value!(y.schema);
        //println!("======\n{}\n======", serde_json::to_string(&sch).unwrap());
        for raw in vec![tnx2, tnx3] {
            //let mut json_deserializer = serde_json::Deserializer::from_str(raw);
            //let () = inferred.deserialize(&mut json_deserializer).unwrap();
            let tnx: RESTTransaction = serde_json::from_str(raw).unwrap();
            let key = format!("{}_{}", "app-1", "http://test.app.com");
            st.update(key.clone(), &tnx).unwrap();
    
        }

        let shape: Shape = inferred.schema.to_json_typegen_shape();
        println!("=++++++\n{:?}\n=+++++", shape);
        let output: String = inferred.schema.process_with_json_typegen(OutputMode::JsonSchema).unwrap();
        println!("=++++++\n{}\n=+++++", output);

        //let res = process_transaction(tnx1).unwrap();
        //assert_eq!(res.source, r"app-1");

    
        let v = st.get(key.as_str()).expect("");
        let output: String = v.resp_schema.as_ref().unwrap().schema.process_with_json_typegen(OutputMode::JsonSchema).unwrap();

        println!("=++++++\n{:#?}\n=+++++", v);
        println!("=++++**\n{}\n=+++**", output);


    }

}