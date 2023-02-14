use std::net::SocketAddr;

use tokio::{net::UdpSocket, select};

use crate::{
    packets::*,
    types::{RaknetBuffer, SizedBuffer},
};
type IResult<T> = Result<T, Box<dyn std::error::Error>>;

async fn proxy(server: SocketAddr) -> IResult<()> {
    let mut client = "0.0.0.0:0".parse().unwrap();
    let conn_server = UdpSocket::bind("0.0.0.0:1888").await?;
    let conn_client = UdpSocket::bind("0.0.0.0:0").await?;
    conn_client.connect(server).await?;
    let mut buf_server = SizedBuffer::<4000>::new();
    let mut buf_client = SizedBuffer::<4000>::new();
    loop {
        select! {
            r = conn_server.recv_from(&mut (*buf_server)[..]) => {
                let (len, addr) = r?;
                *buf_server.len_mut()=len;
                client = addr;
                print!("C->");
                peek(&mut buf_server);
                conn_client.send(buf_server.initialized()?).await?;
                IResult::<()>::Ok(())
            }
            r = conn_client.recv(&mut (*buf_client)[..]) => {
                let len = r?;
                *buf_client.len_mut()=len;
                print!("S->");
                peek(&mut buf_client);
                conn_server.send_to(buf_client.initialized()?, client).await?;
                IResult::<()>::Ok(())
            }
        }?;
        buf_server.clear();
        buf_client.clear();
    }
}

fn peek<const N: usize>(buf: &mut SizedBuffer<N>) {
    let id = PacketId::take(buf);
    println!("get: {:?}", &id);
    if let Ok(id) = id {
        match id {
            PacketId::UConnPing => println!(" \\_ {:?}", UConnPing::take(buf)),
            PacketId::UConnConnPing => println!(" \\_ {:?}", UConnConnPing::take(buf)),
            PacketId::UConnPong => println!(" \\_ {:?}", UConnPong::take(buf)),
            PacketId::OConnReq1 => println!(" \\_ {:?}", OConnReq1::take(buf)),
            PacketId::OConnReply1 => println!(" \\_ {:?}", OConnReply1::take(buf)),
            PacketId::OConnReq2 => println!(" \\_ {:?}", OConnReq2::take(buf)),
            PacketId::OConnReply2 => println!(" \\_ {:?}", OConnReply2::take(buf)),
            PacketId::FrameSet(_) => {
                let f = FrameSet::take(buf);
                if let Ok(f) = f {
                    peek_frame(f)
                } else {
                    println!(" \\_ {:?}", f)
                }
            }
            PacketId::Ack => println!(" \\_ {:?}", Ack::take(buf)),
            PacketId::Nack => println!(" \\_ {:?}", Nack::take(buf)),
            _ => {}
        }
    }
}

fn peek_frame(f: FrameSet) {
    // f.frame.body
}

#[tokio::test]
async fn feature() {
    use tokio::net::lookup_host;
    let server = lookup_host("sg.hivebedrock.network:19132")
        .await
        .unwrap()
        .next()
        .unwrap();
    // let server = "0.0.0.0:1111".parse().unwrap();
    proxy(server).await.unwrap();
}
