#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let grpc_channel = TonicChannel::builder(
    //     "http://127.0.0.1:8777"
    //         .parse()
    //         .expect("Fail to parse server url"),
    // )
    // .connect()
    // .await?;
    // let mut rpc_process_client = RpcProcessClient::new(grpc_channel)
    //     .send_compressed(CompressionEncoding::Gzip)
    //     .accept_compressed(CompressionEncoding::Gzip);
    // let handshake_request = Request::new(HandshakeRequestPacket {
    //     username: "quhao-client".to_string(),
    //     encryption_type: 0,
    //     encryption_token: vec![],
    // });
    // rpc_process_client.handshake(handshake_request).await?;
    // let (request_stream_tx, request_stream_rx) = channel(1024);
    // let request_stream = ReceiverStream::new(request_stream_rx);
    // let response_stream = rpc_process_client.data(request_stream).await?;
    // tokio::spawn(async move {
    //     loop {
    //         if let Err(e) = request_stream_tx
    //             .send(DataPacket {
    //                 session_id: Uuid::new_v4().to_string(),
    //                 encryption_type: EncryptionType::Aes.into(),
    //                 data_packet_type: DataPacketType::DataTransfer.into(),
    //                 encryption_token: vec![],
    //                 packet_payload: vec![],
    //             })
    //             .await
    //         {
    //             eprintln!("Error happen: {e:?}")
    //         };
    //         sleep(Duration::from_secs(120)).await;
    //     }
    // });
    // tokio::spawn(async move {
    //     let mut response_stream = response_stream.into_inner();
    //     loop {
    //         while let Some(Ok(response)) = response_stream.next().await {
    //             println!("Receive response: {response:?}");
    //         }
    //     }
    // });
    // loop {}
    Ok(())
}
