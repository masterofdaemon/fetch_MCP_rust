use fetch_mcp_rust::mcp::stdio::{read_message, write_message};
use tokio::io::duplex;

#[tokio::test]
async fn roundtrip_message() {
    let (mut a, mut b) = duplex(4096);
    let body = br#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
    let writer = async {
        write_message(&mut a, body).await.unwrap();
    };
    let reader = async {
        let got = read_message(&mut b).await.unwrap();
        assert_eq!(got, body);
    };
    tokio::join!(writer, reader);
}
