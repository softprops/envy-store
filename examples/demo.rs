use serde::Deserialize;
use tokio::runtime::Runtime;

// AWS_PROFILE=... aws ssm put-parameter --name /demo/foo --value bar --type SecureString
// AWS_PROFILE=... aws ssm put-parameter --name /demo/bar --value baz,boom,zoom --type StringList
// AWS_PROFILE=... aws ssm put-parameter --name /demo/zar --value 42 --type String
#[derive(Deserialize, Debug)]
struct Config {
    foo: String,
    bar: Vec<String>,
    zar: u32,
}

fn main() {
    let mut rt = Runtime::new().expect("failed to initialize runtime");
    let conf = envy_store::from_path::<Config, _>("/demo");
    println!("config {:#?}", rt.block_on(conf))
}
