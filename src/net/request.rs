use crate::net::protocol::{ProtocolForwardStartArgs, ProtocolArgs, Protocol, ProtocolHeaderType, Data, PROTOCOL_HEAD_VERSION};
use crate::err::{NfResult, NfErrorCode, NfError};
use tokio::io::{AsyncWriteExt, BufReader, BufWriter, AsyncBufReadExt};
use crate::utils::stream::NfBuff;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use crate::net::response::Response;

// ----------------- request ---------------
pub struct Request {}
impl Request {
    // 发送转发开始请求
    pub async fn send_forward_start<T>(writer: &mut T, arg: ProtocolForwardStartArgs) -> NfResult<()>
        where T: AsyncWriteExt + Unpin {
        info!("send forward start request.");
        let args = Some(ProtocolArgs::ForwardStart(arg));
        let mut protocol = Protocol::new(
            ProtocolHeaderType::ForwardStart, args, None);
        Protocol::send(writer, protocol).await
    }

    pub async fn recv_forward_start<T>(reader: &mut T) -> NfResult<ProtocolForwardStartArgs>
        where T: AsyncBufReadExt + Unpin {
        debug!("ready recv forward start response.");
        let proto = Protocol::read(reader).await?;
        debug!("recv protocol: {:?}", &proto);
        let proto_head_version = proto.header.version.clone();
        if proto_head_version != PROTOCOL_HEAD_VERSION {
            return Err(NfError::E(format!("The protocol head version not supported. version: {}", proto_head_version)));
        }
        if proto.header.p_type as u8 != ProtocolHeaderType::ForwardStart as u8 {
            return Err(NfError::E("protocol head type not match. need ForwardStart head type.".to_string()));
        }
        // 连接目标资源
        if proto.args.is_none() {
            return Err(NfError::E(format!("proto args is None.")));
        }
        match proto.args.unwrap() {
            ProtocolArgs::ForwardStart(arg) => {
                Ok(arg)
            },
            _ => return Err(NfError::E(format!("proto args not match.")))
        }
    }

    // 发送转发数据
    pub async fn send_forward_data<T>(writer: &mut T, data: NfBuff) -> NfResult<()>
        where T: AsyncWriteExt + Unpin {
        info!("send forward data request.");
        let mut protocol = Protocol::new(
            ProtocolHeaderType::ForwardStartRes, None, Some(data));
        Protocol::send(writer, protocol).await
    }

    // 请求转发
    pub async fn open_forward_connect(
        target_address: String,
        link_nodes: Vec<String>,
    ) -> NfResult<TcpStream> {
        info!("ready open remote connection. target: {}, link_nodes: {:?}", &target_address, &link_nodes);
        match TcpStream::connect(target_address.as_str()).await {
            Ok(mut socket) => {
                if link_nodes.is_empty() {
                    return Ok(socket);
                }

                // 先发送请求打开连接
                let arg = ProtocolForwardStartArgs{
                    target_address,
                    link_address: link_nodes,
                };
                let (mut reader, mut writer) = socket.split();
                let mut socket_writer = BufWriter::new(writer);
                let mut socket_reader = BufReader::new(reader);

                Request::send_forward_start(&mut socket_writer, arg).await?;
                let data: Data = Response::recv_forward_start(&mut socket_reader).await?;
                if data.code != NfErrorCode::Success as i32 {
                    return Err(NfError::E(data.msg));
                }
                Ok(socket)
            },
            Err(e) => Err(NfError::IoError(format!("connect next address failed.\naddr: {}, err: {}", &target_address, e)))
        }
    }

}