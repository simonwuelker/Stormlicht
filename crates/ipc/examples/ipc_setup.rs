use ipc::{IpcClient, IpcSetupServer};
use std::io;

fn main() -> io::Result<()> {
    env_logger::init();

    let server = IpcSetupServer::create()?;
    let client1 = IpcClient::connect()?;
    let client2 = server.accept()?;

    _ = client1;
    _ = client2;
    Ok(())
}
