use clap::{Arg, ArgMatches, Command};
use lazy_static;
use super::super::utils::convert::StringUtil;
use serde::Serialize;
use futures::AsyncReadExt;
use core::panic;
use std::{io::BufRead, process::exit};
use crate::utils::convert::VecUtil;


// #[cfg(target_os = "unix")]
// pub static DEFAULT_CONFIG_PATH: &str = "./application.yml";
// #[cfg(target_os = "windows")]
// pub static DEFAULT_CONFIG_PATH: &str = ".\\application.yml";
// pub static DEFAULT_LISTEN_ADDRESS: &str = "127.0.0.1:8000";
// #[cfg(target_os = "unix")]
// pub static DEFAULT_LOG_CONFIG_PATH: &str = "./log4rs.yaml";
// #[cfg(target_os = "windows")]
// pub static DEFAULT_LOG_CONFIG_PATH: &str = ".\\log4rs.yaml";

pub static DEFAULT_CONFIG_PATH: &str = "./application.yml";
pub static DEFAULT_LISTEN_ADDRESS: &str = "127.0.0.1:8000";
pub static DEFAULT_LOG_CONFIG_PATH: &str = "./log4rs.yaml";
static DEFAULT_AES_IV: &str = "qwertyuiopASDFGH";


#[derive(Debug, Clone, Serialize)]
pub struct RunClientParam {
    pub link_address: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunServerParam {
    // 监听本地地址
    pub listen_address: String,
    // 指向网络跳转链路。
    pub link_nodes: Vec<String>,    // 可以为空

    // 加解密算法
    pub crypt: SupportCrypt,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rc4Param {
    pub key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AesParam {
    pub key: String,
    pub iv: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum SupportCrypt {
    None,
    Rc4(Rc4Param),
    Aes(AesParam),
}

#[derive(Debug, Clone, Serialize)]
pub struct NfParam {
    pub config: Option<String>,
    pub listen: String,
    pub link_nodes: Vec<String>,
    pub crypt: SupportCrypt,
    pub log_level: String,
}


impl NfParam {
    pub fn get_parse_matches<'a>() -> Command<'a> {

        let app = Command::new("Nf")
            .version("0.1.0")
            .author("l3una. <l3una@outlook.com>")
            .about("Does awesome things")
            .arg(
                Arg::new("LISTEN_ADDRESS")
                    .short('l')
                    .long("listen")
                    .value_name("LISTEN_ADDRESS")
                    .help("listen address.[ip:port]\neg: 127.0.0.1:8000,127.0.0.1:8001")
                    .default_value("127.0.0.1:8000")
                    .required(false)
                    .takes_value(true),
            )
            .arg(
                Arg::new("LINK_NODES")
                    .short('L')
                    .long("link")
                    .value_name("LINK_NODES")
                    .help("forward link node address.[ip:port,ip:port]\neg: 127.0.0.1:8010,127.0.0.1:8011")
                    .required(false)
                    .takes_value(true),
            )
            .arg(
                Arg::new("CRYPT")
                    .short('c')
                    .long("crypt")
                    .value_name("CRYPT")
                    .help("crypt algorithm. [rc4,aes]")
                    .required(false)
                    .takes_value(true)
            )
            .arg(
                Arg::new("KEY")
                    .short('k')
                    .long("key")
                    .value_name("KEY")
                    .help("crypt key.")
                    .required(false)
                    .takes_value(true)
            )
            .arg(
                Arg::new("DEBUG")
                    .short('d')
                    .long("debug")
                    .help("debug")
                    .takes_value(false)
                    .required(false),
            );
        return app;
    }

    pub fn parse() -> Option<Self> {

        let mut app = NfParam::get_parse_matches();

        let app1 = app.clone();
        match NfParam::parse_server(&app1.get_matches()) {
            Some(param) => {
                Some(param)
            },
            None => {
                app.print_help();
                None
            }
        }
    }

    pub fn parse_server(matches: &ArgMatches) -> Option<NfParam> {
        let server = matches;
        let mut link_nodes = vec![];
        match server.value_of("LINK_NODES") {
            Some(s) => {
                let link_str = s.to_string();
                let nodes: Vec<&str> = link_str.split(',').collect();
                link_nodes = VecUtil::str_to_string(nodes);
            },
            None => {}
        }

        let mut level;
        if server.is_present("DEBUG") {
            level = "debug";
        } else {
            level = "warn";
        }
        let crypt_opt = StringUtil::option_str2option_string(server.value_of("CRYPT"));
        let key_opt = StringUtil::option_str2option_string(server.value_of("KEY"));
        let mut crypt = SupportCrypt::None;
        if crypt_opt.is_some(){
            let crypt_value = crypt_opt.unwrap();
            if crypt_value.to_lowercase().eq("rc4") {
                if key_opt.is_some() {
                    let rc4_param = Rc4Param{ key: key_opt.unwrap().clone() };
                    crypt = SupportCrypt::Rc4(rc4_param);
                }
            } else if crypt_value.to_lowercase().eq("aes") {
                if key_opt.is_some() {
                    let iv = DEFAULT_AES_IV.to_string();
                    let k = key_opt.unwrap();
                    if (&k).len() != 32 {
                        println!("the aes key length must 32.");
                        exit(1);
                    }
                    let aes_paam = AesParam{key: k, iv};
                    crypt = SupportCrypt::Aes(aes_paam);
                }
            }
        }
        let mut nf_param = NfParam {
            config: None,
            listen: server.value_of("LISTEN_ADDRESS")?.to_string(),
            link_nodes,
            crypt,
            log_level: level.to_string()
        };
        Some(nf_param)
    }
}