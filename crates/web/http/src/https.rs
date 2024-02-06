use std::{
    net::TcpStream,
    sync::{Arc, OnceLock},
};

static CERTIFICATE_STORE: OnceLock<Arc<rustls::RootCertStore>> = OnceLock::new();

fn root_certificates() -> Arc<rustls::RootCertStore> {
    CERTIFICATE_STORE
        .get_or_init(|| {
            let mut store = rustls::RootCertStore::empty();
            store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            Arc::new(store)
        })
        .clone()
}

pub(crate) fn establish_connection(
    domain_name: String,
) -> Result<rustls::StreamOwned<rustls::ClientConnection, TcpStream>, rustls::Error> {
    let socket = TcpStream::connect((domain_name.as_str(), 443)).expect("Connection failed");
    let server_name = rustls::pki_types::ServerName::try_from(domain_name).expect("invalid domain");

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_certificates())
        .with_no_client_auth();

    let client = rustls::ClientConnection::new(Arc::new(config), server_name)?;
    let stream = rustls::StreamOwned::new(client, socket);
    Ok(stream)
}
