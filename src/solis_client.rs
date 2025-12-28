mod structs;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::Utc;
use hmac::Mac;
use log::debug;
use md5::compute as compute_md5;
use reqwest::blocking::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;

use structs::{
    GetParameterValueRequest, GetParameterValueResponse, InvertorDetailRequest, PagingRequest,
    PagingResponseData, SetParameterValueRequest, SolisResponse,
};
pub use structs::{InverterBriefInfo, InverterCommand, InverterDetailInfo, InverterState};

type HmacSha1 = hmac::Hmac<sha1::Sha1>;

pub struct SolisApi {
    base_url: String,
    api_key_id: String,
    api_key_secret: String,
    client: Client,
}

impl SolisApi {
    pub fn new(base_url: String, api_key_id: String, api_key_secret: String) -> Self {
        Self {
            base_url,
            api_key_id,
            api_key_secret,
            client: Client::new(),
        }
    }

    fn request<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let body_str = serde_json::to_string(body)?;
        debug!("request {} with body: {}", path, &body_str);
        let content_md5 = STANDARD.encode(compute_md5(&body_str).to_vec());
        let content_type = "application/json";
        let now = Utc::now();
        let date = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        let sign_data = format!(
            "POST\n{}\n{}\n{}\n{}",
            content_md5, content_type, date, path
        );
        let mut sign_hmac = HmacSha1::new_from_slice(self.api_key_secret.as_bytes())?;
        sign_hmac.update(sign_data.as_bytes());
        let sign = STANDARD.encode(sign_hmac.finalize().into_bytes());
        let auth = format!("API {}:{}", self.api_key_id, sign);
        let req = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .header("Content-MD5", content_md5)
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .header(reqwest::header::AUTHORIZATION, auth)
            .header("Date", date)
            .body(body_str)
            .build()?;
        let resp_text = self.client.execute(req)?.text()?;
        debug!(
            "response: {}",
            if resp_text.len() > 1000 {
                "..."
            } else {
                &resp_text
            }
        );
        let resp: SolisResponse<T> = serde_json::from_str(&resp_text)?;
        if resp.code != "0" {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                resp.msg.unwrap_or_else(|| "unknown error".to_string()),
            )));
        }
        Ok(resp.data.unwrap())
    }

    pub fn get_inverters(&self) -> Result<Vec<InverterBriefInfo>, Box<dyn std::error::Error>> {
        let request = PagingRequest {
            page_no: 1,
            page_size: 10,
        };
        let resp: PagingResponseData<InverterBriefInfo> =
            self.request("/v1/api/inverterList", &request)?;
        Ok(resp.page.records)
    }

    pub fn get_inverter_detail(
        &self,
        inverter_sn: &str,
    ) -> Result<InverterDetailInfo, Box<dyn std::error::Error>> {
        let request = InvertorDetailRequest { sn: inverter_sn };
        let resp: InverterDetailInfo = self.request("/v1/api/inverterDetail", &request)?;
        Ok(resp)
    }

    pub fn get_inverter_detail_json(
        &self,
        inverter_sn: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let request = InvertorDetailRequest { sn: inverter_sn };
        let resp: serde_json::Value = self.request("/v1/api/inverterDetail", &request)?;
        Ok(resp)
    }

    fn get_parameter_value(
        &self,
        inverter_sn: &str,
        cid: InverterCommand,
    ) -> Result<GetParameterValueResponse, Box<dyn std::error::Error>> {
        let request = GetParameterValueRequest { inverter_sn, cid };
        let resp: GetParameterValueResponse = self.request("/v2/api/atRead", &request)?;
        if resp.need_loop {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Loop responses not supported",
            )))
        } else {
            Ok(resp)
        }
    }

    fn set_parameter_value(
        &self,
        inverter_sn: &str,
        cid: InverterCommand,
        value: &str,
        yuanzhi: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = SetParameterValueRequest {
            inverter_sn,
            cid,
            value,
            yuanzhi,
        };
        let _: serde_json::Value = self.request("/v2/api/control", &request)?;
        Ok(())
    }

    pub fn update_parameter_value_if_needed(
        &self,
        inverter_sn: &str,
        cid: InverterCommand,
        value: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let pv = self.get_parameter_value(inverter_sn, cid.clone())?;
        debug!("parameter value: {:?}", pv);

        if pv.msg == value {
            Ok(false)
        } else {
            self.set_parameter_value(inverter_sn, cid, value, &pv.yuanzhi)?;
            Ok(true)
        }
    }
}
