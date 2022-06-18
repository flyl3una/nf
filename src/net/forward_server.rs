use crate::err::{NfError, NfResult};
use tokio::net::{TcpListener, TcpStream};
use crate::handle::dispatch::Dispatch;
use serde::{Deserialize, Serialize};
use crate::settings::args::SupportCrypt;


#[derive(Debug, Clone)]
pub struct ForwardServer {
    pub listen: String,
    pub link_nodes: Vec<String>,
    pub crypt: SupportCrypt,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForwardServerContext{
    pub link_nodes: Vec<String>,        // 该值存在，则直接将所有数据转发至下一跳地址
    pub crypt: SupportCrypt,
}

pub struct ForwardClientContext {
    // 接收到客户端连接的 socket
    pub socket: TcpStream,
}

impl ForwardServer {

    pub fn new(listen: String, link_nodes: Vec<String>, crypt: SupportCrypt) -> Self {
        Self {
            listen,
            link_nodes,
            crypt
        }
    }

    // 监听地址， 接受到连接进入到新的处理方法中
    // 接收到新连接后，判断是否存在下一跳地址，若存在下一跳地址，则连接下一跳地址
    // 若无下一跳地址，判断是否存在目标地址，若存在目标地址，则连接目标地址，若无目标地址，则自己为目标地址，接收数据并处理。
    // run
    pub async fn run(&self) -> NfResult<()> {
        // 如果下一跳地址存在，则连接下一跳地址
        let link_nodes = self.link_nodes.clone();
        let crypt = self.crypt.clone();
        let server_context = ForwardServerContext{link_nodes, crypt};
        // 监听本地地址
        self.listen(server_context).await
    }

    // 监听地址
    pub async fn listen(&self, context: ForwardServerContext) -> NfResult<()> {
        match TcpListener::bind(&self.listen).await {
            Ok(listener) => {
                info!("listen address: {}", &self.listen);
                loop {
                    match listener.accept().await {
                        Ok((mut socket, peer)) => {
                            info!("accept connect [{}:{}]", peer.ip(), peer.port());

                            let client_context = ForwardClientContext { socket };
                            self.spawn_handle(context.clone(), client_context).await;
                        }
                        Err(e) => return Err(NfError::E(e.to_string())),
                    }
                }
            }
            Err(e) => Err(NfError::E(e.to_string())),
        }
    }

    pub async fn spawn_handle(
        &self,
        server_context: ForwardServerContext,
        client_context: ForwardClientContext,
    ) -> NfResult<()> {
        tokio::spawn(async move {
            // 处理方法
            // handle不能为self的方法。
            if let Err(e) = handle(server_context, client_context).await {
                if let NfError::IoError(w) = &e {
                    warn!("io closed. warn: {}", w);
                    return;
                }
                error!("forward client error: \n{}", e.to_string());
            }
        });
        Ok(())
    }
}


// 处理链接
pub async fn handle(server_context: ForwardServerContext, client_context: ForwardClientContext) -> NfResult<()> {

    let mut handle = Dispatch::new(server_context, client_context);
    Ok(handle.run().await?)
}
