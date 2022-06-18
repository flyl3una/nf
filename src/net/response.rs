use crate::net::protocol::{ProtocolForwardStartArgs, ProtocolArgs, Protocol, ProtocolHeaderType, Data, PROTOCOL_HEAD_VERSION};
use crate::err::{NfResult, NfErrorCode, NfError};
use tokio::io::{AsyncWriteExt, BufReader, BufWriter, AsyncBufReadExt};
use crate::utils::stream::NfBuff;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use std::fmt::Debug;


// ----------------- response --------------
pub struct Response {}
impl Response {
    // 发送转发开始响应
    pub async fn send_forward_start<T>(writer: &mut T, data: Data) -> NfResult<()>
        where
            T: AsyncWriteExt + Unpin,
    {
        debug!("send forward start response.");
        match serde_json::to_string(&data) {
            Ok(buff) => {
                let mut response_proto = Protocol::new(
                    ProtocolHeaderType::ForwardStartRes,
                    Some(ProtocolArgs::Str(buff)), None
                );
                Protocol::send(writer, response_proto).await
            },
            Err(e) => return Err(NfError::ConvertError(format!("data convert json failed. data: {:?}", data)))
        }
    }

    // 读取转发开始响应数据
    pub async fn recv_forward_start<T>(reader: &mut T) -> NfResult<Data>
    where T: AsyncBufReadExt + Unpin {
        debug!("recv forward start response.");
        let protocol = Protocol::read(reader).await?;
        if protocol.header.version != PROTOCOL_HEAD_VERSION ||
            protocol.header.p_type as u8 != ProtocolHeaderType::ForwardStartRes as u8 {
            return Err(NfError::E(format!("recv forward start protocol response failed.")));
        }
        match protocol.args {
            Some(arg) => {
                match arg {
                    ProtocolArgs::Str(s) => {
                        match serde_json::from_str::<Data>(&s) {
                            Ok(t) => Ok(t),
                            Err(e) => return Err(NfError::ConvertError(format!("recv forward start protocol response arg convert object failed. err: {}", e)))
                        }
                    },
                    _ => return Err(NfError::E(format!("recv forward start response arg not matched.")))
                }
            },
            None => {
                return Err(NfError::E(format!("recv forward start protocol arg is None.")));
            }
        }
    }
}
