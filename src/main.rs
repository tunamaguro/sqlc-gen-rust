use prost::Message as _;
use std::io::{Read as _, Write};

pub mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

fn deserialize_codegen_request(data: &[u8]) -> Result<plugin::GenerateRequest, prost::DecodeError> {
    plugin::GenerateRequest::decode(data)
}

fn serialize_codegen_response(response: &plugin::GenerateResponse) -> Vec<u8> {
    response.encode_to_vec()
}

fn main() {
    let mut stdin = std::io::stdin().lock();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer).unwrap();

    let _request = deserialize_codegen_request(&buffer).expect("Failed to decode GenerateRequest");

    let file = plugin::File {
        name: "hello.txt".to_string(),
        contents: "Hello, world!".as_bytes().to_vec(),
    };
    let mut response = plugin::GenerateResponse::default();
    response.files.push(file);

    let serialized_response = serialize_codegen_response(&response);

    std::io::stdout()
        .write_all(&serialized_response)
        .expect("Failed to write response");
}
