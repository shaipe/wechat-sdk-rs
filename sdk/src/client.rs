//! copyright © shaipe 2020 - persent
//! 微信对接的网络请求客户端

use crate::errors::WeChatError;
use crate::WeChatResult;
use bytes::Bytes;
use reqwest::header;
use reqwest::Client as HttpClient;
use serde::Serialize;
use std::time::Duration;

const DEFAULT_USER_AGENT: &'static str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/70.0.3534.4 Safari/537.36";

/// 请求客户端
pub struct Client {
    pub(crate) client: HttpClient,
}

impl Client {
    pub fn new() -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(DEFAULT_USER_AGENT),
        );

        Client {
            client: HttpClient::builder()
                .timeout(Duration::from_secs(300))
                .connect_timeout(Duration::from_secs(300))
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }
    /// post方式提交数据
    /// url:
    /// param:
    pub async fn post<T: Serialize + ?Sized>(&self, url: &str, params: &T) -> WeChatResult<String> {
        match self.client.post(url).json(params).send().await {
            Ok(res) => {
                if res.status() == 200 {
                    match res.text().await {
                        Ok(txt) => {
                            // println!("--- {} ----", txt);
                            Ok(txt)
                        }
                        Err(e) => Err(WeChatError::ClientError {
                            errcode: -1,
                            errmsg: format!("Send request error: {}", e),
                        }),
                    }
                } else {
                    Err(WeChatError::ClientError {
                        errcode: 500,
                        errmsg: format!("status={}", res.status()),
                    })
                }
            }
            Err(e) => Err(WeChatError::ClientError {
                errcode: 500,
                errmsg: format!("Send request error: {}", e),
            }),
        }
    }

    /// 发送二进制文件
    pub async fn post_betyes(&self, url: &str, body: Bytes) -> WeChatResult<String> {
        match self.client.post(url).body(body).send().await {
            Ok(res) => {
                if res.status() == 200 {
                    match res.text().await {
                        Ok(txt) => Ok(txt),
                        Err(e) => Err(WeChatError::ClientError {
                            errcode: -1,
                            errmsg: format!("Send request error: {}", e),
                        }),
                    }
                } else {
                    Err(WeChatError::ClientError {
                        errcode: 500,
                        errmsg: format!("status={}", res.status()),
                    })
                }
            }
            Err(e) => Err(WeChatError::ClientError {
                errcode: 500,
                errmsg: format!("Send request error: {}", e),
            }),
        }
    }

    /// get方法
    #[allow(dead_code)]
    pub async fn get(&self, url: &str) -> WeChatResult<String> {
        match self.client.get(url).send().await {
            Ok(res) => {
                if res.status() == 200 {
                    match res.text().await {
                        Ok(txt) => Ok(txt),
                        Err(e) => Err(WeChatError::ClientError {
                            errcode: -1,
                            errmsg: format!("Send request error: {}", e),
                        }),
                    }
                } else {
                    Err(WeChatError::ClientError {
                        errcode: 500,
                        errmsg: format!("status={}", res.status()),
                    })
                }
            }
            Err(e) => Err(WeChatError::ClientError {
                errcode: 500,
                errmsg: format!("Send request error: {}", e),
            }),
        }
    }

    ///
    #[inline]
    pub fn json_decode(&self, data: &str) -> WeChatResult<serde_json::Value> {
        let obj: serde_json::Value = match serde_json::from_str(data) {
            Ok(decoded) => decoded,
            Err(ref e) => {
                return Err(WeChatError::ClientError {
                    errcode: -3,
                    errmsg: format!("Json decode error: {}", e),
                });
            }
        };
        let errcode = match obj["errcode"].as_i64() {
            Some(v) => v,
            None => 0,
        };
        if errcode != 0 {
            let errmsg: String = obj["errmsg"].to_string();
            return Err(WeChatError::ClientError {
                errcode: errcode as i32,
                errmsg: errmsg,
            });
        }
        println!("obj====={:?}", obj);
        Ok(obj)
    }
}