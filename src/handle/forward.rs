use crate::err::{NfResult, NfError};
use crate::net::protocol::{ProtocolForwardStartArgs, Protocol, ProtocolHeader, PROTOCOL_HEAD_VERSION, ProtocolHeaderType};
use tokio::net::TcpStream;
use crate::utils::stream::NfBuff;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
use crate::utils::stream::StreamUtil;
use crate::net::request::Request;
use crate::net::response::Response;
use crate::settings::args::SupportCrypt;
use crate::utils::crypt::{rc4_encrypt, rc4_decrypt, aes_encrypt, aes_decrypt};


pub struct ForwardHandle {
    // pub args: ProtocolForwardStartArgs
}

impl ForwardHandle {
    // 网络数据透传 source -> target ， target -> source
    pub async fn empty_to_empty<R, W>(
        source_socket_reader: &mut R,
        source_socket_writer: &mut W,
        target_socket_reader: &mut R,
        target_socket_writer: &mut W) -> NfResult<()>
        where
            W: AsyncWriteExt + Unpin,
            R: AsyncBufReadExt + Unpin,
    {
        loop {
            tokio::select! {
                // 接收目标资源协议数据
                recv_data_rst = StreamUtil::read_all(target_socket_reader) => {
                    // debug!("recv response empty data.");
                    let recv_buff: NfBuff = recv_data_rst?;
                    StreamUtil::write_all(source_socket_writer, recv_buff).await?;
                }
                send_data_rst = StreamUtil::read_all(source_socket_reader) => {
                    // debug!("recv forward empty data.");
                    let send_buff: NfBuff = send_data_rst?;
                    StreamUtil::write_all(target_socket_writer, send_buff).await?;
                }
            }
        }
        Ok(())
    }


    // 从源地址接收协议数据，并封包，发送协议数据至目标地址。
    pub async fn empty_to_proto<R, W>(
        source_socket_reader: &mut R,
        source_socket_writer: &mut W,
        target_socket_reader: &mut R,
        target_socket_writer: &mut W,
        crypt_info: SupportCrypt
    ) -> NfResult<()>
        where
            W: AsyncWriteExt + Unpin,
            R: AsyncBufReadExt + Unpin,
    {
        loop {
            tokio::select! {
                // 接收目标资源协议数据
                res_proto_rst = Protocol::read(target_socket_reader) => {
                    let res: Protocol = res_proto_rst?;
                    if res.header.version != PROTOCOL_HEAD_VERSION {
                        return Err(NfError::E(format!("protocol version not supported.")));
                    }
                    match res.header.p_type {
                        ProtocolHeaderType::ForwardDataRes => {
                            match res.body {
                                Some(data) => {
                                    let data_buff = match (crypt_info.clone()) {
                                        SupportCrypt::Rc4(rc4_param) => {
                                            rc4_encrypt(&data[..], rc4_param.key.clone())?
                                        },
                                        SupportCrypt::Aes(aes_param) => {
                                            aes_decrypt(&data[..], aes_param.key.clone(), aes_param.iv.clone())?
                                        }
                                        _ => data,
                                    };
                                    StreamUtil::write_all(source_socket_writer, data_buff).await?
                                },
                                None => return Err(NfError::E(format!("protocol body is None."))),
                            }
                        },
                        ProtocolHeaderType::ForwardEndRes => {
                            // TODO: 关闭连接
                        },
                        _ => {
                            return Err(NfError::E(format!("protocol head not supported.")));
                        }
                    }
                    ()
                }
                // 接收来源地址透明数据
                send_data_rst = StreamUtil::read_all(source_socket_reader) => {
                    let send_data = send_data_rst?;

                    let data_buff = match (crypt_info.clone()) {
                        SupportCrypt::Rc4(rc4_param) => {
                            rc4_encrypt(&send_data[..], rc4_param.key.clone())?
                        },
                        SupportCrypt::Aes(aes_param) => {
                            aes_encrypt(&send_data[..], aes_param.key.clone(), aes_param.iv.clone())?
                        }
                        _ => send_data,
                    };
                    let send_protocol = Protocol::new(ProtocolHeaderType::ForwardData, None, Some(data_buff));
                    Protocol::send(target_socket_writer, send_protocol).await?
                }
            }
        }
        Ok(())
    }

    // 从目标地址接收协议数据，并解包，返回原始数据至原地址。
    pub async fn proto_to_empty<R, W>(
        source_socket_reader: &mut R,
        source_socket_writer: &mut W,
        target_socket_reader: &mut R,
        target_socket_writer: &mut W,
        mut crypt_info: SupportCrypt) -> NfResult<()>
        where
            W: AsyncWriteExt + Unpin,
            R: AsyncBufReadExt + Unpin,
    {
        loop {
            tokio::select! {
                // 接收源地址 协议数据
                res_proto_rst = Protocol::read(source_socket_reader) => {
                    let res: Protocol = res_proto_rst?;
                    if res.header.version != PROTOCOL_HEAD_VERSION {
                        return Err(NfError::E(format!("protocol version not supported.")));
                    }
                    match res.header.p_type {
                        ProtocolHeaderType::ForwardData => {
                            match res.body {
                                Some(data) => {
                                    let data_buff = match crypt_info.clone() {
                                        SupportCrypt::Rc4(rc4_param) => {
                                            let key = rc4_param.key.clone();
                                            rc4_encrypt(&data[..], key)?
                                        },
                                        SupportCrypt::Aes(aes_param) => {
                                            let key = aes_param.key.clone();
                                            aes_decrypt(&data[..], key, aes_param.iv.clone())?
                                        }
                                        _ => data,
                                    };
                                    StreamUtil::write_all(target_socket_writer, data_buff).await?
                                },
                                None => return Err(NfError::E(format!("protocol body is None."))),
                            }
                        },
                        ProtocolHeaderType::ForwardEnd => {
                            // TODO: 关闭连接
                        },
                        _ => {
                            return Err(NfError::E(format!("protocol head not supported.")));
                        }
                    }
                    ()
                }
                // 接收目标地址透明数据
                send_data_rst = StreamUtil::read_all(target_socket_reader) => {
                    let send_data: NfBuff = send_data_rst?;
                    let data_buff = match crypt_info.clone() {
                        SupportCrypt::Rc4(rc4_param) => {
                            let key = rc4_param.key.clone();
                            rc4_encrypt(&send_data[..], key.clone())?
                        },
                        SupportCrypt::Aes(aes_param) => {
                            aes_encrypt(&send_data[..], aes_param.key.clone(), aes_param.iv.clone())?
                        }
                        _ => send_data,
                    };
                    let send_protocol = Protocol::new(ProtocolHeaderType::ForwardDataRes, None, Some(data_buff));
                    Protocol::send(source_socket_writer, send_protocol).await?
                }
            }
        }
    }

    // 从目标地址接收协议数据，并解包转发给目标地址。
    pub async fn proto_to_proto<R, W>(
        source_socket_reader: &mut R,
        source_socket_writer: &mut W,
        target_socket_reader: &mut R,
        target_socket_writer: &mut W) -> NfResult<()>
        where
            W: AsyncWriteExt + Unpin,
            R: AsyncBufReadExt + Unpin,
    {
        loop {
            tokio::select! {
                // 接收源地址 协议数据
                res_proto_rst = Protocol::read(source_socket_reader) => {
                    let res: Protocol = res_proto_rst?;
                    Protocol::send(target_socket_writer, res).await?;
                }
                // 接收目标地址透明数据
                send_data_rst = StreamUtil::read_all(target_socket_reader) => {
                    let send_data: NfBuff = send_data_rst?;
                    Request::send_forward_data(source_socket_writer, send_data).await?;
                }
            }
        }
    }
}