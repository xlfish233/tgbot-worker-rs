
use std::borrow::Cow;
use serde::Serialize;

#[derive(Serialize)]
pub struct WrappedResponse<'a> {
    code: u16,
    msg: Cow<'a, str>,
    detail: Option<Cow<'a, str>>,
}

pub fn wrapped_response<'a>(code: u16, msg: &'a str, detail: Option<&'a str>) -> String {
    let response = WrappedResponse {
        code,
        msg: Cow::Borrowed(msg),
        detail: detail.map(Cow::Borrowed),
    };
    serde_json::to_string(&response).unwrap()
}
