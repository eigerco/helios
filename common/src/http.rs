use crate::errors::HttpError;
use ic_cdk::api::management_canister::http_request::HttpResponse;

pub async fn http_get(url: &str) -> Result<HttpResponse, HttpError> {
    let req = ic_http::create_request().get(url).build();
    // TODO: should we pass cycles in http_get method or we should have a default one?
    let resp = ic_http::http_request(req, 2_603_101_200).await?;
    Ok(resp.0)
}
