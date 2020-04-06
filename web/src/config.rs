use serde_derive::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Mutex;
use wechat_sdk::tripartite::{TripartiteConfig,set_tripartite_config};
// 业务配置信息
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub tripartite: Option<TripartiteConfig>,
}

impl Config {
    // 加载配置
    pub fn init(config_path: &str) -> Self {
        let file_path =  if config_path.is_empty() {
            "prod.conf"
        }
        else{
            config_path
        };

        // 打开文件
        let mut file = match File::open(file_path) {
            Ok(f) => f,
            Err(e) => panic!("no such file {} exception: {}", file_path, e)
        };

        // 读取文件到字符串变量
        let mut str_val = String::new();
        match file.read_to_string(&mut str_val) {
            Ok(s) => s,
            Err(e) => panic!("Error Reading file:{}", e)
        };
       
        let cnf: Config = serde_json::from_str(&str_val).unwrap();
        match cnf.clone().tripartite{
            Some(val) => {
                set_tripartite_config(val.clone());
                val
            },
            _ => {
                println!("请配置第三方文件!");
                TripartiteConfig::default()
            }
        };
        cnf
    }
}
