use bytes::{BytesMut, BufMut};
use tokio::time::Duration;
use crate::err::NfError;
use crate::err::NfResult;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};


pub type NfBuff = Vec<u8>;
pub static NF_BUFF_LEN: usize = 1024 * 1024;


pub struct StreamUtil {}

impl StreamUtil {

    //
    /// ```no_run
    /// use tokio::io::{BufReader, BufWriter};
    /// use tokio::net::{TcpStream};
    /// use tokio::net::tcp::{WriteHalf, ReadHalf};
    /// let mut socket = TcpStream::connect("127.0.0.1:8000").await?;
    /// let (mut reader, mut writer) = socket.split();
    /// let mut socket_reader: BufReader<ReadHalf> = BufReader::new(reader);
    /// let mut socket_writer: BufWriter<WriteHalf> = BufWriter::new(writer);
    /// // 注意buff长度最大为NF_BUFF_LEN，超出长度需要多次读取。
    /// let buf = read_all(socket_reader).await;
    ///
    /// ```
    /// ReaderHalf
    pub async fn read_all<T>(stream: &mut T) -> NfResult<NfBuff>
    where
        T: AsyncBufReadExt + Unpin,
    {
        // let buff_len = 5;
        let buff_len = NF_BUFF_LEN;
        let mut buf: Vec<u8> = vec![];
        // let mut buf: Vec<u8> = vec![0u8; buff_len];
        // let mut buff: Vec<u8> = vec![0u8; buff_len];
        let mut buff = BytesMut::with_capacity(buff_len);

        // let mut buff = Vec::new();
        // read_buf 读取到0时表示结束。
        match stream.read_buf(&mut buff).await {
            Ok(n) => {
                if n == 0{
                    return Err(NfError::IoError("reader stream break.".to_string()));
                }
                debug!("recv length: {}", &n);
                return Ok(buff[..n].to_vec());
            }
            Err(e) => return Err(NfError::IoError(e.to_string())),
        }
        // loop {
        //     // let mut buff = vec![0u8; buff_len];
        //     let mut buff = BytesMut::with_capacity(buff_len);
        //
        //     match stream.read_buf(&mut buff).await {
        //         Ok(n) => {
        //             if n == 0 {
        //                 break;
        //             }
        //             debug!("recv len: {}", &n);
        //             buf.put_slice(&buff[..n]);
        //             // buf.extend_from_slice(&buff[..n]);
        //             if n != buff_len {
        //                 break;
        //             }
        //         }
        //         Err(e) => return Err(NfError::IoError(e.to_string())),
        //     }
        // }
        // if buf.len() == 0 {
        //     let err = format!("ready buff failed from stream. len = 0.");
        //     return Err(NfError::E(err));
        // }
        // Ok(buf)
    }

    //
    /// ```no_run
    /// use tokio::io::{BufReader, BufWriter};
    /// use tokio::net::{TcpSocket, TcpStream};
    /// let mut socket = TcpStream::connect("127.0.0.1:8000").await?;
    /// let (mut reader, mut writer) = socket.split();
    /// let mut socket_reader = BufReader::new(reader);
    /// let mut socket_writer = BufWriter::new(writer);
    /// let buf = read_exact(socket_reader, 1024).await;
    ///
    /// ```
    /// ReaderHalf
    pub async fn read_exact<T>(stream: &mut T, length: usize) -> NfResult<NfBuff>
    where
        T: AsyncBufReadExt + Unpin,
    {
        let mut buff = vec![0u8; length];
        // let mut buff = BytesMut::with_capacity(length);
        // let mut buff = Vec::new();
        match stream.read_exact(buff.as_mut_slice()).await {
            Ok(n) => {
                // debug!("read exact buff:{:?}", &buff);
                Ok(buff.to_vec())
            },
            Err(e) => Err(NfError::IoError(e.to_string())),
        }
    }

    //
    /// ```no_run
    /// use tokio::io::{BufReader, BufWriter};
    /// use tokio::net::{TcpStream};
    /// use tokio::net::tcp::{WriteHalf, ReadHalf};
    /// let mut socket = TcpStream::connect("127.0.0.1:8000").await?;
    /// let (mut reader, mut writer) = socket.split();
    /// let mut socket_reader: BufReader<ReadHalf> = BufReader::new(reader);
    /// let mut socket_writer: BufWriter<WriteHalf> = BufWriter::new(writer);
    /// // 注意buff长度最大为NF_BUFF_LEN，超出长度需要多次读取。
    /// let buff: Vec<u8> = vec![1,2,3];
    /// let buf = write_all(socket_writer, buff).await;
    /// ```
    // OwnedWriteHalf
    pub async fn write_all<T>(stream: &mut T, buff: NfBuff) -> NfResult<()> 
    where T: AsyncWriteExt + Unpin {
        match stream.write_all(&buff[..]).await
         {
            Err(e) => Err(NfError::IoError(e.to_string())),
            _ => {
                // debug!("send buff successful. buff: {:?}", buff);
                stream.flush().await;
                Ok(())
            }
        }
    }
}