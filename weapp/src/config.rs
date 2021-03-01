//! copyright © shaipe 2021 - present
//! 小程序后端接口对接配置信息
//! created by shaipe 20210228

use std::fs::File;

/// 配置信息结构体
pub struct Config {
    // 应用id
    pub app_id: String,
    // 应用密钥
    pub secret: String,
    // 小程序名称
    pub name: String,
}

impl Config {
 
    /// 加载yml配置文件
    pub fn load_yaml(conf_path: &str) -> WechatResult<Config> {
        use yaml_rust::yaml;
        // open file
        let mut f = match File::open(conf_path) {
            Ok(f) => f,
            Err(e) => {
                return Err(error! {
                    code: 4004,
                    msg: format!("{}", e)
                });
            }
        };
        let mut s = String::new();
        match f.read_to_string(&mut s) {
            Ok(s) => s,
            Err(e) => {
                return Err(error! {
                    code: 4004,
                    msg: format!("Error Reading file: {}", e)
                });
            }
        };
        // f.read_to_string(&mut s).unwrap(); // read file content to s
        // load string to yaml loader
        let docs = yaml::YamlLoader::load_from_str(&s).unwrap();
        // get first yaml hash doc
        let yaml_doc = &docs[0];
        // get server value
        let server = yaml_doc["web"].clone();

        Ok(Config::load_yaml_node(&server))
    }

    /// 根据yaml配置点进加载配置
    /// @param1: yaml 配置节点
    pub fn load_yaml_node(conf_node: &yaml_rust::yaml::Yaml) -> Config {
        Config {
            ip: if let Some(s) = conf_node["ip"].as_str() {
                s.to_owned()
            } else {
                "0.0.0.0".to_owned()
            },
            port: if let Some(p) = conf_node["port"].as_i64() {
                p as u64
            } else {
                8080
            },
            static_dir: if let Some(s) = conf_node["static_dir"].as_str() {
                s.to_owned()
            } else {
                "static".to_owned()
            },
            static_ext: vec![],
            debug: Some(false), //server["debug"]. ,
            cert_path: None,
            tls: None,
        }
    }
}

impl std::default::Default for Config {
    // 给定默认值
    fn default() -> Config {
        Config {
            ip: "0.0.0.0".to_owned(),
            port: 9090,
            static_dir: "www".to_owned(),
            static_ext: vec![
                ".html".to_owned(),
                ".js".to_owned(),
                ".png".to_owned(),
                ".jpg".to_owned(),
                ".css".to_owned(),
            ],
            debug: Some(false),
            cert_path: None,
            tls: None,
        }
    }
}
