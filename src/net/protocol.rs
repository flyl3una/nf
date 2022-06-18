use crate::utils::stream::StreamUtil;
use tokio::io::{BufReader, BufWriter, AsyncBufReadExt, AsyncWriteExt};
use crate::err::{NfResult, NfError};
use std::convert::{From, Into};
use std::prelude::rust_2021::{TryFrom, TryInto};
use crate::utils::stream::NfBuff;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use tokio::net::TcpStream;
use crate::err::NfErrorCode::Success;
use crate::err::NfErrorCode;


#[derive(Debug, Clone, Serialize)]
pub struct Protocol {
    pub header: ProtocolHeader,
    pub args: Option<ProtocolArgs>,
    pub body: Option<NfBuff>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct ProtocolHeader {
    #[serde(rename = "type")]
    // 协议版本
    pub version: u8,
    // 协议类型, u8
    pub p_type: ProtocolHeaderType,       // response时，type加0x100
    // 协议扩展参数长度
    pub args_len: u32,

    // 协议body长度
    pub body_len: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
// #[repr(C)]
// back_to_enum! {
pub enum ProtocolHeaderType {
    None = 0,
    ForwardStart,
    // i32
    ForwardData,
    ForwardEnd,

    ForwardStartRes = 0x81,
    ForwardDataRes,
    ForwardEndRes,
}

pub const PROTOCOL_HEAD_VERSION: u8 = 0x01;
pub const NF_RESPONSE_TYPE_INCREASE: u8 = 0x01;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolForwardStartArgs {
    // 协议参数,json 格式，根据协议定
    pub target_address: String,
    pub link_address: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolArgs {
    // 协议参数,json 格式，根据协议定
    Str(String),
    ForwardStart(ProtocolForwardStartArgs),
    // Data(Data),
}

// #[derive(Debug, Clone, Serialize, Default)]
// pub struct ProtocolArgs {
//     // 协议参数,json 格式，根据协议定
//     pub args: String,
// }

#[derive(Debug, Clone, Serialize, Default)]
pub struct ProtocolBody {
    pub body: Vec<u8>,
}

pub const NF_PROTOCOL_HEAD_LENGTH: usize = 14;     // 根据Protocolheader计算得出

impl Into<u8> for ProtocolHeaderType {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for ProtocolHeaderType {
    fn from(value: u8) -> Self {
        use ProtocolHeaderType::*;
        if value == ForwardStart as u8{
            ForwardStart
        } else if value == ForwardEnd as u8 {
            ForwardEnd
        } else if value == ForwardData as u8 {
            ForwardData
        } else if value == ForwardDataRes as u8 {
            ForwardDataRes
        } else if value == ForwardEndRes as u8 {
            ForwardEndRes
        } else if value == ForwardStartRes as u8 {
            ForwardStartRes
        } else {
            None
        }
    }
}

impl From<[u8; NF_PROTOCOL_HEAD_LENGTH]> for ProtocolHeader {
    fn from(buff: [u8; NF_PROTOCOL_HEAD_LENGTH]) -> Self {
        let err = format!("protocol buff index error. buff len: {}", NF_PROTOCOL_HEAD_LENGTH);
        // let type_buff: [u8; 4] = buff[1..5].try_into().expect(err.as_str());
        let arg_len_buff: [u8; 4] = buff[2..6].try_into().expect(err.as_str());
        let body_len_buff: [u8; 8] = buff[6..14].try_into().expect(err.as_str());
        // let p_type_num = u32::from_be_bytes(type_buff);
        let args_len = u32::from_be_bytes(arg_len_buff);
        let body_len = u64::from_be_bytes(body_len_buff);
        Self {
            version: buff[0],
            p_type: ProtocolHeaderType::from(buff[1]),
            args_len,
            body_len,
        }
    }
}

impl Into<Vec<u8>> for ProtocolHeader {
    fn into(self) -> Vec<u8> {
        let mut buff: Vec<u8> = vec![];

        buff.push(self.version);
        buff.push(self.p_type as u8);
        // let p_type_num: u8 = self.p_type as u8;
        // buff.extend_from_slice(&p_type_num.to_ne_bytes());
        buff.extend_from_slice(&self.args_len.to_be_bytes());
        buff.extend_from_slice(&self.body_len.to_be_bytes());
        buff
    }
}

impl TryFrom<Vec<u8>> for ProtocolHeader {
    type Error = NfError;

    fn try_from(buff: Vec<u8>) -> Result<Self, Self::Error> {
        let length = buff.len();
        if length.clone() < NF_PROTOCOL_HEAD_LENGTH {
            return Err(NfError::E(format!("the protocol buff must is {}, current length: {}", NF_PROTOCOL_HEAD_LENGTH, &length)));
        }

        let err = format!("protocol buff index error. buff len: {}", length);


        // let type_buff = buff[1..5].try_into().unwrap();
        let arg_len_buff: [u8; 4] = buff[2..6].try_into().unwrap();
        let body_len_buff: [u8; 8] = buff[6..14].try_into().unwrap();

        // let p_type_num = u32::from_be_bytes(type_buff);
        let args_len = u32::from_be_bytes(arg_len_buff);
        let body_len = u64::from_be_bytes(body_len_buff);
        // ProtocolHeaderType.
        // let header_type: ProtocolHeaderType = p_type_num.into();
        let header_type = ProtocolHeaderType::from(buff[1]);
        let header = ProtocolHeader {
            version: buff[0],
            p_type: header_type,
            args_len,
            body_len,
        };
        Ok(header)
    }
}



impl ProtocolArgs {
    pub fn len(&self) -> usize {
        match self {
            ProtocolArgs::Str(s) => s.len(),
            ProtocolArgs::ForwardStart(arg) => {
                let buff = serde_json::to_vec(&arg).unwrap();
                buff.len()
            }
        }
    }
}

impl Protocol {
    // 接收协议
    pub async fn read<T>(reader: &mut T) -> NfResult<Self>
        where T: AsyncBufReadExt + Unpin,
    {
        let mut proto_args = None;
        let mut proto_body = None;
        let proto_header_buff = StreamUtil::read_exact(reader, NF_PROTOCOL_HEAD_LENGTH).await?;

        let proto_header = ProtocolHeader::try_from(proto_header_buff)?;
        let args_len = proto_header.args_len.clone() as usize;
        if args_len != 0 {
            let proto_args_buff = StreamUtil::read_exact(reader, args_len).await?;
            let origin_bytes = format!("bytes: {:?}", &proto_args_buff);
            let args_rst = String::from_utf8(proto_args_buff);
            if args_rst.is_err() {
                return Err(NfError::E(format!("convert string failed. origin: {}", origin_bytes)));
            }
            let args_string = args_rst.unwrap();

            proto_args = match proto_header.p_type {
                ProtocolHeaderType::None => Some(ProtocolArgs::Str(args_string)),
                ProtocolHeaderType::ForwardStart => {
                    match serde_json::from_slice(args_string.as_bytes()) {
                        Ok(arg) => {
                            Some(ProtocolArgs::ForwardStart(arg))
                        },
                        Err(e) => {
                            return Err(NfError::E(format!("parse protocol arg failed. e: {}", e)));
                        }
                    }
                },
                ProtocolHeaderType::ForwardData => {
                    Some(ProtocolArgs::Str(args_string))
                },
                ProtocolHeaderType::ForwardEnd => Some(ProtocolArgs::Str(args_string)),
                ProtocolHeaderType::ForwardStartRes => Some(ProtocolArgs::Str(args_string)),
                ProtocolHeaderType::ForwardDataRes => Some(ProtocolArgs::Str(args_string)),
                _ => {return Err(NfError::E(format!("not support protocol head type. type")))}
            }
        }
        let body_len = proto_header.body_len.clone() as usize;
        if proto_header.body_len != 0 {
            let proto_body_buff = StreamUtil::read_exact(reader, body_len).await?;
            proto_body = Some(proto_body_buff );
            // proto_body = Some(proto_body);
        }
        let nf_proto = Protocol {
            header: proto_header,
            args: proto_args,
            body: proto_body,
        };
        Ok(nf_proto)
    }


    //发送数据
    pub async fn send<T>(writer: &mut T, proto: Protocol) -> NfResult<()>
        where T: AsyncWriteExt + Unpin {
        debug!("ready send protocol: {:?}", &proto);
        let header_buff: Vec<u8> = proto.header.into();
        match StreamUtil::write_all(writer, header_buff).await {
        // match writer.write_all(&header_buff[..]).await {
            Err(e) => {return Err(NfError::E(format!("writer header failed.")));},
            _ => {},
        }
        if let Some(args) = proto.args {
            let arg_str = match args{
                ProtocolArgs::ForwardStart(f) => {
                    match serde_json::to_string(&f) {
                        Ok(t) => t,
                        Err(e) => return Err(NfError::E(format!("serde protocol args buff failed.")))
                    }
                },
                ProtocolArgs::Str(s) => s
            };
            debug!("send protocol args: {}", arg_str);
            let args_buff = arg_str.as_bytes().to_vec();
            match StreamUtil::write_all(writer, args_buff).await {
            // match writer.write_all(&args_buff[..]).await {
                Err(e) => {return Err(NfError::E(format!("writer protocol args buff failed.")));},
                _ => {},
            }
        }
        if let Some(body_buff) = proto.body {
            // let body_buff = body.body;
            match StreamUtil::write_all(writer, body_buff).await {
            // match writer.write_all(&body_buff[..]).await {
                Err(e) => {return Err(NfError::E(format!("writer protocol body buff failed.")));},
                _ => {},
            }
        }
        // writer.flush().await;
        Ok(())
    }

    // 生成一个相应数据
    pub fn new(p_type: ProtocolHeaderType, args: Option<ProtocolArgs>, body: Option<NfBuff>) -> Self {
        let mut body_len = 0u64;
        let mut args_len = 0u32;
        if body.is_some() {
            body_len = body.clone().unwrap().len() as u64;
        }
        if args.is_some() {
            args_len = args.clone().unwrap().len() as u32;
        }

        Self{
            header: ProtocolHeader {
                version: PROTOCOL_HEAD_VERSION,
                p_type,
                args_len,
                body_len,
            },
            args,
            body
        }
    }
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Data {
    pub code: i32,
    pub msg: String,
    pub data: serde_json::Value,
}


impl Data {
    pub fn update_error<E>(&mut self, code: i32, msg: E) where E: ToString {
        self.code = code;
        self.msg = msg.to_string();
    }
}


