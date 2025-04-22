use crate::{Error, Result};

use reqwest::{
    header::{HeaderMap, HeaderValue},
    Response, StatusCode,
};

pub(crate) fn header_map<T>(pairs: T) -> HeaderMap
where
    T: IntoIterator<Item = (&'static str, &'static str)>,
{
    let mut map = HeaderMap::new();
    for (key, value) in pairs {
        map.insert(key, HeaderValue::from_static(value));
    }
    map
}

pub(crate) fn map_reqwest_response(res: reqwest::Result<Response>) -> Result<Response> {
    match res.and_then(|res| res.error_for_status()) {
        Ok(res) => Ok(res),
        Err(err) => match err.status() {
            Some(StatusCode::UNAUTHORIZED) => Err(Error::ApiTokenInvalid),
            Some(StatusCode::NOT_FOUND) => Err(Error::NotFound),
            _ => Err(Error::Reqwest(err)),
        },
    }
}
