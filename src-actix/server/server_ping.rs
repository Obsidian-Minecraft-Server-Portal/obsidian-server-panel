use crate::server::server_data::ServerData;
use anyhow::Result;
use craftping::tokio::ping;
use craftping::Response;
use tokio::net::TcpStream;

impl ServerData {
    pub async fn get_ping(&self) -> Result<Response> {
        let server_port = self.get_server_properties()?.server_port.ok_or(anyhow::anyhow!("Server port not found"))? as u16;
        let hostname = "localhost";
        let mut stream = TcpStream::connect((hostname, server_port)).await?;
        let response = ping(&mut stream, hostname, server_port).await?;
        Ok(response)
    }
}
