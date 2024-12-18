use crate::peer::info::PeerInfo;
use crate::peer::{Peer, Status};
use crate::rpc::request::{Ping, ReadRequest, Request, WriteRequest};
use log::{error, info};
use std::io;
use std::net::SocketAddr;
use std::process::exit;
use std::time::SystemTime;
use tokio::net::TcpStream;

pub async fn connect<A>(addr: A, peer_info: &mut PeerInfo)
where
	A: Into<SocketAddr>,
{
	let addr = addr.into();
	let Ok(mut stream) = TcpStream::connect(addr).await else {
		error!("peer at {addr} is unreachable");
		exit(1);
	};

	let ping = Ping::new(peer_info.id, peer_info.addr, peer_info.chat_addr);
	if let Err(e) = stream.write_req(ping).await {
		error!("failed to send ping to peer at {addr}: {e}");
		exit(1);
	}

	let pong = match stream.read_req(1024).await {
		Ok(Request::Pong(pong)) => pong,
		Ok(_) => {
			error!("unexpected response from peer at {addr} (not a pong)");
			exit(1);
		}
		Err(e) if e.kind() == io::ErrorKind::ConnectionAborted => {
			error!("peer at {addr} aborted connection");
			exit(1);
		}
		Err(e) => {
			error!("failed to receive pong from peer at {addr}: {e}");
			exit(1);
		}
	};

	let peer = peer_info.peers.entry(pong.peer_id).or_insert(Peer::new(
		pong.peer_id,
		addr,
		pong.peer_chat_addr,
	));
	peer.status = Status::Online;
	peer.last_seen = Some(SystemTime::now());

	if let Err(e) = peer_info.save().await {
		error!("failed to save peer info: {e}");
		exit(1);
	}

	info!("connected to peer at {addr}");
}
