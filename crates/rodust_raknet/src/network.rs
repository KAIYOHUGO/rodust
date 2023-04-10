use std::{io::Result, net::SocketAddr};

use tokio::{net::UdpSocket, select};

#[derive(Debug)]
pub struct MITM<C, S> {
    to_client: C,
    to_server: S,
}

impl<C, S> MITM<C, S>
where
    C: Fn(&mut [u8], usize) -> Option<&[u8]>,
    S: Fn(&mut [u8], usize) -> Option<&[u8]>,
{
    pub fn new(to_client: C, to_server: S) -> Self {
        Self {
            to_client,
            to_server,
        }
    }
    /// `S---Conn_C===Conn_S---C`
    pub async fn proxy<const B: usize>(self, server: SocketAddr, port: u16) -> Result<()> {
        let mut client = "0.0.0.0:0".parse().expect("should not fail");
        let conn_server = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
        let conn_client = UdpSocket::bind("0.0.0.0:0").await?;
        conn_client.connect(server).await?;
        let mut buf_server = [0u8; B];
        let mut buf_client = [0u8; B];
        loop {
            select! {
                r = conn_server.recv_from(&mut buf_server) => {
                    let (len, addr) = r?;
                    client = addr;
                    if let Some(buf) = (self.to_server)(&mut buf_server, len) {
                        conn_client.send(buf).await?;
                    }
                    Result::<()>::Ok(())
                }
                r = conn_client.recv(&mut buf_client) => {
                    let len = r?;
                    if let Some(buf) = (self.to_client)(&mut buf_client, len) {
                        conn_server.send_to(buf, client).await?;
                    }
                    Result::<()>::Ok(())
                }
            }?;
        }
    }
}
