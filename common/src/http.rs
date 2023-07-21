use crate::errors::CanisterCallError;
use ic_cdk::api::management_canister::http_request::HttpResponse;

pub async fn http_get(url: &str) -> Result<HttpResponse, CanisterCallError> {
    let req = ic_http::create_request().get(url).build();
    let resp = ic_http::http_request(req, 2_603_101_200).await?;
    Ok(resp.0)
}
