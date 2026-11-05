use super::*;

async fn bound() -> (Arc<TcpListener>, SocketAddr) {
    let listener = Arc::new(TcpListener::bind("127.0.0.1:0").await.unwrap());
    let address = listener.local_addr().unwrap();
    (listener, address)
}

#[tokio::test]
async fn it_reports_the_address_of_the_socket_it_shares() {
    let (listener, address) = bound().await;
    assert_eq!(SharedListener(listener).local_addr().unwrap(), address);
}

#[tokio::test]
async fn it_accepts_a_connection_and_names_its_peer() {
    let (listener, address) = bound().await;
    let client = TcpStream::connect(address).await.unwrap();
    let (_, peer) = SharedListener(listener).accept().await;
    assert_eq!(peer, client.local_addr().unwrap());
}

#[tokio::test]
async fn a_later_generation_accepts_on_the_same_socket_the_earlier_one_used() {
    let (listener, address) = bound().await;
    let mut first = SharedListener(listener.clone());
    let _first_client = TcpStream::connect(address).await.unwrap();
    first.accept().await;
    drop(first);

    let mut second = SharedListener(listener);
    let second_client = TcpStream::connect(address).await.unwrap();
    let (_, peer) = second.accept().await;
    assert_eq!(peer, second_client.local_addr().unwrap());
}
