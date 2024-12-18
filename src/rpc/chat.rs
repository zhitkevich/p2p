use crate::peer::info::PeerInfo;
use crate::rpc::request::{Message, ReadRequest, Request, WriteRequest};
use crossterm::terminal;
use log::error;
use std::collections::{HashMap, VecDeque};
use std::process::exit;
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task;

pub async fn start(peer_info: &PeerInfo) {
	let (tx, rx) = mpsc::channel(32);
	let tx_clone = tx.clone();
	let peer_info_clone = peer_info.clone();
	task::spawn(async move { handle_input(tx_clone, &peer_info_clone).await });
	task::spawn(handle_output(rx));
	listen(tx, peer_info).await;
}

async fn handle_input(tx: mpsc::Sender<Message>, peer_info: &PeerInfo) {
	let mut streams = HashMap::new();
	for (id, peer) in &peer_info.peers {
		let Ok(stream) = TcpStream::connect(peer.chat_addr).await else { continue };
		streams.insert(id, stream);
	}
	let mut stdin = BufReader::new(stdin());
	let mut input = String::new();

	loop {
		stdin.read_line(&mut input).await.unwrap();
		let msg = Message::new(peer_info.id, input.trim());
		tx.send(msg.clone()).await.unwrap();

		for stream in streams.values_mut() {
			let _ = stream.write_req(msg.clone()).await;
		}
		input.clear();
	}
}

async fn handle_output(mut rx: mpsc::Receiver<Message>) {
	let mut stdout = stdout();
	let mut lines = VecDeque::new();
	let size = terminal::size().unwrap();
	let max_width = size.0 as usize;
	let max_height = size.1 as usize;

	stdout.write_all(b"\x1b[2J\x1b[H").await.unwrap();

	loop {
		let title_line = format!("\x1b[H\x1b[48;5;255m\x1b[30m{:^max_width$}\x1b[0m", "p2p / chat");
		stdout.write_all(format!("{title_line}\x1b[{max_height};0H> ").as_bytes()).await.unwrap();
		stdout.flush().await.unwrap();

		let msg = rx.recv().await.unwrap();
		lines.push_front(format!("{}: {}", msg.peer_id, msg.text));
		if lines.len() > max_height - 2 {
			lines.pop_back();
		}

		stdout.write_all(b"\x1b[2J\x1b[H").await.unwrap();
		for (i, line) in lines.iter().enumerate() {
			let height = max_height - i - 1;
			stdout.write_all(format!("\x1b[{height};1H{line}").as_bytes()).await.unwrap();
		}
	}
}

async fn listen(tx: mpsc::Sender<Message>, peer_info: &PeerInfo) {
	let listener = TcpListener::bind(&peer_info.chat_addr).await.unwrap_or_else(|e| {
		error!("failed to start chat listener on {}: {e}", peer_info.chat_addr);
		exit(1);
	});
	while let Ok((mut stream, _)) = listener.accept().await {
		loop {
			let Ok(Request::Message(msg)) = stream.read_req(1024).await else { break };
			tx.send(msg).await.unwrap();
		}
	}
}
