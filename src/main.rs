#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_must_use)]

pub mod settings;
pub mod logger;
pub mod utils;
pub mod err;
pub mod net;
pub mod handle;
mod tests;


#[macro_use]
extern crate log;
#[macro_use]
extern crate tokio;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate anyhow;
extern crate crypto;


use crate::logger::init_console_log;
use std::process::exit;
use err::{NfResult, NfError};
use settings::args::{NfParam, RunServerParam};
// use crate::logger::init_log;
use crate::net::forward_server::{ForwardServer};
use crate::err::anyhow_error_to_chain;



pub fn init(run_args: &NfParam) {
    // &configs::args::ARGS;
    // let arg = configs::args::ARGS.clone();

    init_console_log(run_args.log_level.clone());
    // init_log(run_args.config.clone()).unwrap();
    debug!("arg: {:?}", run_args);
    // &configs::setting::APP_CONFIG;
    // let a = &services::SERVICE_CONTEXT.redis_service;
    debug!("application init successful.");
}

async fn run(run_args: &NfParam) -> NfResult<()> {
    /* let arg = configs::args::ARGS.clone(); */
    let run_arg = run_args.clone();
    let server_param = RunServerParam {
        listen_address: run_arg.listen,
        link_nodes: run_arg.link_nodes,
        shell: run_arg.shell,
        crypt: run_args.crypt.clone(),
    };
    server(server_param).await
}


async fn server(param: RunServerParam) -> NfResult<()> {

    let nf_server = ForwardServer::new(param.listen_address, param.link_nodes, param.shell, param.crypt);
    nf_server.run().await
}

#[tokio::main]
async fn main() {
    //     解析参数
    match NfParam::parse() {
        Some(run_args) => {
            init(&run_args);
            match run(&run_args).await {
                Ok(_) => {
                    exit(0)
                },
                Err(e) => {
                    error!("error: {}", &e);
                    exit(1);
                }
            }
        },
        None => {
            // anyhow_error_to_chain(&e);
            println!("the param format error.")
        }
    }

}
