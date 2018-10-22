//! Envy store provides a means to resolve a collection of [AWS Parameter Store](https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-paramstore.html)
//! values at runtime required for an application to run and deserialize them into a type safe struct.
//!
//! The idea here is that applications that may have previously used the [12-factor practice](https://12factor.net/config)
//! of storing configuration in environment variables, perhaps deserializing them using [envy](https://crates.io/crates/envy),
//! are now configured using the same pattern but resolving values from AWS Parameter Store instead
//!
//! This crate assumes you are using the AWS best practice of [storing related parameters under
//! a prefixed hierarchy](https://aws.amazon.com/blogs/mt/organize-parameters-by-hierarchy-tags-or-amazon-cloudwatch-events-with-amazon-ec2-systems-manager-parameter-store/).
//! This leads to better clarity on what application a set of parameters belong to as well as enables
//! the paths based query API which has performance benefits and is the recommended best practice by AWS.
//!
//! This crate assumes the use of the [AWS default credential chain](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-getting-started.html) for authenticating requests
//! with AWS. Don't worry, if you've used any AWS tooling in the past, you likely already have this configured.
//! You will also need to ensure these credentials have the `ssm:GetParametersByPath` [IAM permission](https://docs.aws.amazon.com/systems-manager/latest/userguide/sysman-paramstore-access.html).
//!
//! # Example
//!
//! ```rust,norun
//! extern crate envy_store;
//! #[macro_use]
//! extern crate serde_derive;
//!
//! /// Type resolvable by prefixed parameter store values
//! /// aws ssm put-parameter --name /demo/foo --value bar --type SecureString
//! /// aws ssm put-parameter --name /demo/bar --value baz,boom,zoom --type StringList
//! /// aws ssm put-parameter --name /demo/zar --value 42 --type String
//! #[derive(Deserialize)]
//! struct Config {
//!   foo: String,
//!   bar: Vec<String>,
//!   zar: u32,
//! }
//!
//! fn main() {
//!    // Returns a `Future` containing the result of a deserialized `Config` type
//!    let config = envy_store::from_path::<Config, _>(
//!      "/demo"
//!    );
//! }
//! ```
#![deny(missing_docs)]
extern crate envy;
extern crate futures;
extern crate rusoto_ssm;
extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate maplit;
#[cfg(test)]
extern crate rusoto_mock;
#[cfg(test)]
extern crate serde_json;

mod error;

// Std lib
use std::collections::HashMap;
use std::path::Path;

// Third party

use futures::{stream, Future, Stream};
use rusoto_ssm::{GetParametersByPathRequest, Parameter, Ssm, SsmClient};
use serde::de::DeserializeOwned;

// Ours

pub use error::Error;

/// Resolves parameter store values and deserialize them into
/// a typesafe struct
///
/// `path_prefix` is assumed to be the path prefixed, e.g `/sweet-app/prod`.
/// Parameter store value names are then expected be of the form `/sweet-app/prod/db-pass`
/// `/sweet-app/prod/db-username`, and so forth.
pub fn from_path<T, P>(path_prefix: P) -> impl Future<Item = T, Error = Error> + Send
where
    T: DeserializeOwned + Send,
    P: AsRef<Path>,
{
    ::from_client(SsmClient::new(Default::default()), path_prefix)
}

/// Resolves parameter store values and deserializes them into
/// a typesafe struct. Similar to [from_path](fn.from_path.html) but
/// also accepts a customized `rusoto_ssm::Ssm`
/// implementation
pub fn from_client<T, C, P>(
    client: C,
    path_prefix: P,
) -> impl Future<Item = T, Error = Error> + Send
where
    T: DeserializeOwned + Send,
    C: Ssm + Send,
    P: AsRef<Path>,
{
    enum PageState {
        Start(Option<String>),
        Next(String),
        End,
    }
    let prefix = path_prefix
        .as_ref()
        .to_str()
        .unwrap_or_default()
        .to_string();
    let prefix_strip = prefix.len() + 1;
    stream::unfold(PageState::Start(None), move |state| {
        let next_token = match state {
            PageState::Start(start) => start,
            PageState::Next(next) => Some(next),
            PageState::End => return None,
        };
        Some(
            client
                .get_parameters_by_path(GetParametersByPathRequest {
                    next_token,
                    path: prefix.clone(),
                    with_decryption: Some(true),
                    recursive: Some(true),
                    ..GetParametersByPathRequest::default()
                })
                .map_err(Error::from)
                .map(move |resp| {
                    let next_state = match resp.next_token {
                        Some(next) => {
                            if next.is_empty() {
                                PageState::End
                            } else {
                                PageState::Next(next)
                            }
                        }
                        _ => PageState::End,
                    };
                    (
                        stream::iter_ok(resp.parameters.unwrap_or_default()),
                        next_state,
                    )
                }),
        )
    })
    .flatten()
    .collect()
    .and_then(move |parameters| deserialize(prefix_strip, parameters))
}

fn deserialize<T>(
    prefix_strip: usize,
    parameters: Vec<Parameter>,
) -> Result<T, Error>
where
    T: DeserializeOwned + Send,
{
    envy::from_iter::<_, T>(parameters.into_iter().fold(
        HashMap::new(),
        |mut result: HashMap<String, String>, param| {
            if let (Some(name), Some(value)) = (param.name, param.value) {
                result.insert(name[prefix_strip..].to_string(), value);
            }
            result
        },
    ))
    .map_err(Error::from)
}

#[cfg(test)]
mod tests {

    use super::*;
    use rusoto_mock::{MockCredentialsProvider, MockRequestDispatcher};

    #[test]
    fn deserializes_from_client() {
        let mock = MockRequestDispatcher::with_status(200).with_body(
            r#"{
            "Parameters": [
                { "Name": "/test/foo", "Value": "bar" }
            ]
        }"#,
        );

        assert_eq!(
            Ok(hashmap!("foo".to_string() => "bar".to_string())),
            from_client::<HashMap<String, String>, _, _>(
                SsmClient::new_with(mock, MockCredentialsProvider, Default::default()),
                "/test",
            )
            .wait()
        )
    }

    #[test]
    fn deserializes_with_expected_parameters() {
        let parameters = vec![Parameter {
            name: Some("/test/foo".into()),
            value: Some("bar".into()),
            ..Parameter::default()
        }];
        assert_eq!(
            Ok(hashmap!("foo".to_string() => "bar".to_string())),
            deserialize(6, parameters)
        )
    }
}
