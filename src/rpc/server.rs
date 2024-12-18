use crate::peer::info::PeerInfo;
use crate::peer::Status;
use crate::rpc::request::{Ping, Pong, ReadRequest, Request, WriteRequest};
use log::{error, warn};
use std::process::exit;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task;

pub async fn listen(peer_info: &PeerInfo) {
	let listener = TcpListener::bind(peer_info.addr).await.unwrap_or_else(|e| {
		error!("failed to start server listener on {}: {e}", peer_info.addr);
		exit(1);
	});
	let peer_info = Arc::new(Mutex::new(peer_info.clone()));
	while let Ok((mut stream, _)) = listener.accept().await {
		let peer_info_clone = Arc::clone(&peer_info);
		task::spawn(async move { handle(&mut stream, &peer_info_clone).await });
	}
}

async fn handle(stream: &mut TcpStream, peer_info: &Arc<Mutex<PeerInfo>>) {
	loop {
		let Ok(Request::Ping(req)) = stream.read_req(1024).await else { continue };
		handle_ping(stream, &req, peer_info).await;
	}
}

async fn handle_ping(stream: &mut TcpStream, req: &Ping, peer_info: &Arc<Mutex<PeerInfo>>) {
	let mut peer_info = peer_info.lock().await;
	if stream.write_req(Pong::new(peer_info.id, peer_info.chat_addr)).await.is_err() {
		warn!("peer that sent ping at {} is unreachable", req.peer_addr);
		return;
	}

	let peer = peer_info.peer_or_insert(req.peer_id, req.peer_addr, req.peer_chat_addr);
	peer.status = Status::Online;
	peer.last_seen = Some(SystemTime::now());

	if let Err(e) = peer_info.save().await {
		error!("failed to save peer info: {e}");
	}
}
