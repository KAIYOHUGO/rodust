use std::net::SocketAddr;

use tokio::{net::UdpSocket, select};
use zeco::Deserialize;

use crate::zeco_packets::*;
type IResult<T> = Result<T, Box<dyn std::error::Error>>;

async fn proxy(server: SocketAddr) -> IResult<()> {
    let mut client = "0.0.0.0:0".parse().unwrap();
    let conn_server = UdpSocket::bind("0.0.0.0:1888").await?;
    let conn_client = UdpSocket::bind("0.0.0.0:0").await?;
    conn_client.connect(server).await?;
    // let mut buf_server = SizedBuffer::<4000>::new();
    // let mut buf_client = SizedBuffer::<4000>::new();
    let mut buf_server = [0u8; 4000];
    let mut offset_server = 0;
    let mut buf_client = [0u8; 4000];
    let mut offset_client = 0;
    loop {
        select! {
            r = conn_server.recv_from(&mut buf_server) => {
                let (len, addr) = r?;
                let buf=&buf_server[..len];
                client = addr;
                print!("C->");
                peek(buf,&mut offset_server)?;
                conn_client.send(buf).await?;
                IResult::<()>::Ok(())
            }
            r = conn_client.recv(&mut buf_client) => {
                let len = r?;
                let buf=&buf_client[..len];
                print!("S->");
                peek(buf,&mut offset_client)?;
                conn_server.send_to(buf, client).await?;
                IResult::<()>::Ok(())
            }
        }?;
        offset_server = 0;
        offset_client = 0;
    }
}

fn peek(buf: &[u8], offset: &mut usize) -> IResult<()> {
    let id = PacketId::deserialize(buf, offset, ());
    println!("get: {:?}", &id);
    if let Ok(id) = id {
        match id {
            // PacketId::UConnPing => println!(" \\_ {:?}", UConnPing::deserialize(buf, offset, ())?),
            // PacketId::UConnConnPing => {
            //     println!(" \\_ {:?}", UConnConnPing::deserialize(buf, offset, ())?)
            // }
            // PacketId::UConnPong => println!(" \\_ {:?}", UConnPong::deserialize(buf, offset, ())?),
            // PacketId::OConnReq1 => println!(" \\_ {:?}", OConnReq1::deserialize(buf, offset, ())?),
            // PacketId::OConnReply1 => {
            //     println!(" \\_ {:?}", OConnReply1::deserialize(buf, offset, ())?)
            // }
            // PacketId::OConnReq2 => println!(" \\_ {:?}", OConnReq2::deserialize(buf, offset, ())?),
            // PacketId::OConnReply2 => {
            //     println!(" \\_ {:?}", OConnReply2::deserialize(buf, offset, ())?)
            // }
            PacketId::FrameSet => {
                let frame_set = FrameSet::deserialize(buf, offset, ())?;
                // println!(" \\_ {:?}", &frame_set);
                let mut frame_offset = 0;
                if let Some(fragment) = frame_set.frame.fragment {
                    if fragment.index != 0 {
                        println!("  \\_ [fragment]",);
                        return Ok(());
                    }
                }
                let id = FramePacketId::deserialize(frame_set.frame.body, &mut frame_offset, ())?;
                println!("  \\_ {:?}", &id);
                // match id {
                //     FramePacketId::ConnReq => todo!(),
                //     FramePacketId::ConnReqAccept => todo!(),
                //     FramePacketId::ConnPing => todo!(),
                //     FramePacketId::ConnPong => todo!(),
                //     FramePacketId::NewConn => todo!(),
                //     FramePacketId::DisConn => todo!(),
                //     FramePacketId::Game => todo!(),
                // }
            }
            // PacketId::Ack => println!(" \\_ {:?}", Ack::deserialize(buf, offset, ())),
            // PacketId::Nack => println!(" \\_ {:?}", Nack::deserialize(buf, offset, ())),
            _ => {}
        }
    }
    Ok(())
}

// fn peek<const N: usize>(buf: &mut SizedBuffer<N>) {
//     let id = PacketId::take(buf);
//     println!("get: {:?}", &id);
//     if let Ok(id) = id {
//         match id {
//             PacketId::UConnPing => println!(" \\_ {:?}", UConnPing::take(buf)),
//             PacketId::UConnConnPing => println!(" \\_ {:?}", UConnConnPing::take(buf)),
//             PacketId::UConnPong => println!(" \\_ {:?}", UConnPong::take(buf)),
//             PacketId::OConnReq1 => println!(" \\_ {:?}", OConnReq1::take(buf)),
//             PacketId::OConnReply1 => println!(" \\_ {:?}", OConnReply1::take(buf)),
//             PacketId::OConnReq2 => println!(" \\_ {:?}", OConnReq2::take(buf)),
//             PacketId::OConnReply2 => println!(" \\_ {:?}", OConnReply2::take(buf)),
//             PacketId::FrameSet(_) => {
//                 let f = FrameSet::take(buf);
//                 if let Ok(f) = f {
//                     peek_frame(f)
//                 } else {
//                     println!(" \\_ {:?}", f)
//                 }
//             }
//             PacketId::Ack => println!(" \\_ {:?}", Ack::take(buf)),
//             PacketId::Nack => println!(" \\_ {:?}", Nack::take(buf)),
//             _ => {}
//         }
//     }
// }

// fn peek_frame(f: FrameSet) {
//     // f.frame.body
// }

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
