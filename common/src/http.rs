// Values are taken from https://internetcomputer.org/docs/current/developer-docs/gas-cost
// for a 13-node app subnet.
pub const HTTP_OCALL_BASE_COST: u128 = 49_140_000;
pub const HTTP_OCALL_REQ_BYTE_COST: u128 = 5_200;
pub const HTTP_OCALL_RESP_BYTE_COST: u128 = 10_400;

// TODO: should we pass cycles in http_x methods or we should have a default one?
pub const DEFAULT_HTTP_OUTCALL_COST: u128 = HTTP_OCALL_BASE_COST
    + 5 * 1024 * HTTP_OCALL_REQ_BYTE_COST
    + 5 * 1024 * HTTP_OCALL_RESP_BYTE_COST;

pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
pub use icp::{get, post};

#[cfg(not(target_arch = "wasm32"))]
pub use native::{get, post};

#[cfg(target_arch = "wasm32")]
mod icp {
    use ic_cdk::api::{
        call::CallResult,
        management_canister::http_request::{
            http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
            HttpResponse as CanisterHttpResponse,
        },
    };

    use crate::errors::HttpError;

    use super::{HttpResponse, DEFAULT_HTTP_OUTCALL_COST};

    pub async fn get(url: &str) -> Result<HttpResponse, HttpError> {
        let req = CanisterHttpRequestArgument {
            url: url.to_owned(),
            method: HttpMethod::GET,
            ..Default::default()
        };
        let resp = http_request(req, DEFAULT_HTTP_OUTCALL_COST).await?;
        resp.0.try_into()
    }

    pub async fn post(
        url: &str,
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> Result<HttpResponse, HttpError> {
        let headers = headers
            .iter()
            .map(|(name, value)| HttpHeader {
                name: name.to_string(),
                value: value.to_string(),
            })
            .collect();
        let req = CanisterHttpRequestArgument {
            url: url.to_string(),
            method: HttpMethod::POST,
            headers,
            body: Some(body),
            ..Default::default()
        };
        let resp = http_request(req, DEFAULT_HTTP_OUTCALL_COST).await?;
        resp.0.try_into()
    }

    impl TryFrom<CanisterHttpResponse> for HttpResponse {
        type Error = HttpError;

        fn try_from(value: CanisterHttpResponse) -> Result<Self, Self::Error> {
            let status = value.status.0.try_into().map_err(|err| {
                HttpError::Other(format!("Status should be a 3 digit number, got: {err}"))
            })?;

            Ok(Self {
                status,
                body: value.body,
            })
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

    use crate::errors::HttpError;

    use super::HttpResponse;

    pub async fn get(url: &str) -> Result<HttpResponse, HttpError> {
        let resp = reqwest::get(url)
            .await
            .map_err(|err| HttpError::Other(format!("Request failed: {err}")))?;
        HttpResponse::from_reqwest(resp).await
    }

    pub async fn post(
        url: &str,
        headers: &[(&str, &str)],
        body: Vec<u8>,
    ) -> Result<HttpResponse, HttpError> {
        let mut header_map = HeaderMap::new();
        for &(name, value) in headers {
            let name: HeaderName = name
                .parse()
                .map_err(|_| HttpError::Other(format!("Invalid header name: {name}")))?;
            let value: HeaderValue = value
                .parse()
                .map_err(|_| HttpError::Other(format!("Invalid header value: {value}")))?;
            header_map.insert(name, value);
        }

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .body(body)
            .headers(header_map)
            .send()
            .await
            .map_err(|err| HttpError::Other(format!("Request failed: {err}")))?;

        HttpResponse::from_reqwest(resp).await
    }

    impl HttpResponse {
        async fn from_reqwest(resp: reqwest::Response) -> Result<Self, HttpError> {
            let status = resp.status().as_u16();
            let body = resp
                .bytes()
                .await
                .map_err(|err| HttpError::Other(format!("Couldn't read response body: {err}")))?;

            Ok(Self {
                status,
                body: body.into(),
            })
        }
    }
}
