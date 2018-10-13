extern crate envy;
extern crate futures;
extern crate rusoto_ssm;
extern crate serde;

use std::collections::HashMap;

use futures::{stream, Future, Stream};
use rusoto_ssm::{GetParametersByPathError, GetParametersByPathRequest, Ssm};
use serde::de::DeserializeOwned;

enum Error {
  Store(GetParametersByPathError),
  Envy(envy::Error),
}

impl From<GetParametersByPathError> for Error {
  fn from(err: GetParametersByPathError) -> Self {
    Error::Store(err)
  }
}

impl From<envy::Error> for Error {
  fn from(err: envy::Error) -> Self {
    Error::Envy(err)
  }
}

pub fn from_client<T, C>(client: C, path_prefix: String) -> impl Future<Item = T, Error = Error>
where
  T: DeserializeOwned,
  C: Ssm,
{
  enum PageState {
    Start(Option<String>),
    Next(String),
    End,
  }
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
          path: path_prefix.clone(),
          ..GetParametersByPathRequest::default()
        }).map_err(Error::from)
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
  }).flatten()
  .collect()
  .and_then(|parameters| {
    envy::from_iter::<_, T>(
      parameters
        .into_iter()
        .fold(
          HashMap::new(),
          |mut result: HashMap<String, String>, param| {
            if let (Some(name), Some(value)) = (param.name, param.value) {
              result.insert(name, value);
            }
            result
          },
        ).into_iter(),
    ).map_err(Error::from)
  })
}
