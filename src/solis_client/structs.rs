use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PagingRequest {
    pub page_no: u32,
    pub page_size: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SolisResponse<T> {
    pub code: String,
    pub msg: Option<String>,
    pub data: Option<T>,
}

#[derive(Deserialize)]
pub(super) struct PagingResponseDataPage<T> {
    pub records: Vec<T>,
}

#[derive(Deserialize)]
pub(super) struct PagingResponseData<T> {
    pub page: PagingResponseDataPage<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InverterBriefInfo {
    pub sn: String,
    pub id: String,
}

#[derive(Serialize)]
pub(super) struct InvertorDetailRequest<'a> {
    pub sn: &'a str,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq)]
#[repr(u8)]
pub enum InverterState {
    Online = 1,
    Offline = 2,
    Alert = 3,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InverterDetailInfo {
    pub state: InverterState,
    pub battery_percent: f64,
    pub u_ac1: f64,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum InverterCommand {
    Time = 56,
    AllowGridCharging = 109,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GetParameterValueRequest<'a> {
    pub inverter_sn: &'a str,
    pub cid: InverterCommand,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct GetParameterValueResponse {
    pub msg: String,
    pub yuanzhi: String,
    #[serde(deserialize_with = "deserialize_string_as_bool")]
    pub need_loop: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SetParameterValueRequest<'a> {
    pub inverter_sn: &'a str,
    pub cid: InverterCommand,
    pub value: &'a str,
    pub yuanzhi: &'a str,
}

fn deserialize_string_as_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<bool>().map_err(D::Error::custom)
}
