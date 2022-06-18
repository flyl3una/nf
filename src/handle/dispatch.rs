use tokio::time::Duration;
use tokio::io::{BufReader, BufWriter, AsyncReadExt, AsyncWriteExt, AsyncWrite, AsyncBufReadExt};
use crate::net::protocol::{Protocol, ProtocolHeaderType, ProtocolArgs, ProtocolForwardStartArgs, ProtocolHeader, PROTOCOL_HEAD_VERSION, Data};
use crate::utils::stream::StreamUtil;
use crate::net::forward_server::{ForwardServerContext, ForwardClientContext};
use crate::err::NfResult;
use tokio::net::{TcpStream, TcpSocket};
use crate::handle::forward::{ForwardHandle};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use crate::utils::stream::NfBuff;
use anyhow::private::kind::TraitKind;
use crate::net::protocol::ProtocolHeaderType::ForwardStartRes;
use crate::net::request::Request;
use crate::net::response::Response;
use crate::err::NfError;

pub struct Dispatch {
    server_context: ForwardServerContext,
    client_context: ForwardClientContext,
}

impl Dispatch {
    pub fn new(server_context: ForwardServerContext, client_context: ForwardClientContext) -> Self {
        Self {
            server_context,
            client_context,
        }
    }

    // 获取
    pub async fn connect_target_from_forward_start_request(&mut self) -> NfResult<(TcpStream, Vec<String>)> {
        debug!("ready connect target address from request.");
        let (mut reader, mut writer) = self.client_context.socket.split();
        let mut socket_writer = BufWriter::new(writer);
        let mut socket_reader = BufReader::new(reader);

        let arg = Request::recv_forward_start(&mut socket_reader).await?;

        let mut res_data = Data::default();
        let link_nodes = arg.link_address;
        if link_nodes.is_empty() {
            res_data.update_error(1, "the link nodes is None.")
        }
        let target_addr = link_nodes[0].clone();
        let next_link_nodes = link_nodes[1..].to_vec();
        match Request::open_forward_connect(target_addr.clone(), next_link_nodes.clone()).await {
            Ok(s) => {
                Response::send_forward_start(&mut socket_writer, res_data).await?;
                return Ok((s, next_link_nodes));
            }
            Err(e) => {
                res_data.update_error(3, format!("connect remote address failed. \naddress: {}, err: {}", &target_addr, e.to_string()))
            }
        }

        let err = res_data.msg.clone();
        Response::send_forward_start(&mut socket_writer, res_data).await?;
        Err(NfError::E(err))
    }

    pub async fn run_listen_protocol(
        &mut self,
    ) -> NfResult<()>
    {
        // 接收协议
        let (mut target_socket, next_link_nodes) = self.connect_target_from_forward_start_request().await?;
        let mut target_stream = &mut target_socket;
        let (mut target_reader, mut target_writer) = target_stream.split();
        let mut target_socket_reader = BufReader::new(target_reader);
        let mut target_socket_writer = BufWriter::new(target_writer);

        let (mut reader, mut writer) = self.client_context.socket.split();
        let mut socket_writer = BufWriter::new(writer);
        let mut socket_reader = BufReader::new(reader);

        // 转发数据
        if next_link_nodes.is_empty() {
            // empty -> empty
            ForwardHandle::proto_to_empty(
                &mut socket_reader, &mut socket_writer,
                &mut target_socket_reader, &mut target_socket_writer,
            self.server_context.crypt.clone()).await
        } else {
            ForwardHandle::proto_to_proto(
                &mut socket_reader, &mut socket_writer,
                &mut target_socket_reader, &mut target_socket_writer).await
        }

    }


    pub async fn run_link_forward(
        &mut self,
        link_nodes: Vec<String>) -> NfResult<()> {
        if link_nodes.is_empty() {
            return Err(NfError::E(format!("the link nodes is None.")));
        }

        // 先连接目标地址
        let next_address = link_nodes[0].clone();
        // 如果new_link为空，表名下一跳即为目标地址。
        let new_link = link_nodes[1..].to_vec();
        debug!("connect next address: {}", next_address.as_str());
        let mut target_socket = Request::open_forward_connect(next_address, new_link.clone()).await?;

        let source_socket = &mut self.client_context.socket;
        let (mut source_reader, mut source_writer) = source_socket.split();
        let mut source_socket_reader = BufReader::new(source_reader);
        let mut source_socket_writer = BufWriter::new(source_writer);

        let target_stream = &mut target_socket;
        let (mut target_reader, mut target_writer) = target_stream.split();
        let mut target_socket_writer = BufWriter::new(target_writer);
        let mut target_socket_reader = BufReader::new(target_reader);
        if new_link.is_empty() {
            ForwardHandle::empty_to_empty(
                &mut source_socket_reader, &mut source_socket_writer,
                &mut target_socket_reader, &mut target_socket_writer).await?;
        } else {
            ForwardHandle::empty_to_proto(
                &mut source_socket_reader, &mut source_socket_writer,
                &mut target_socket_reader, &mut target_socket_writer,
                self.server_context.crypt.clone()
            ).await?;
        }

        // 包装为ProtocolHeadData 协议
        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        debug!("dispatch connection...");

        // 透传
        if self.server_context.link_nodes.is_empty() {
            self.run_listen_protocol().await?
        } else {
            let link = self.server_context.link_nodes.clone();
            self.run_link_forward(link).await?
        }
        self.client_context.socket.shutdown();
        debug!("connection end.");

        Ok(())
    }
}