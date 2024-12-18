use crate::args::{gen_completion, Args, Command, ConnectArgs};
use crate::conf::Conf;
use crate::crypto::Uuid;
use crate::peer::info::PeerInfo;
use crate::peer::Peer;
use clap::Parser;
use log::error;
use openssl::rsa::Rsa;
use std::collections::HashMap;
use std::process::exit;
use std::time::Duration;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;

mod args;
mod conf;
mod crypto;
mod peer;
mod rpc;

#[tokio::main]
async fn main() {
	env_logger::builder().format_timestamp(None).format_target(false).init();

	let args = Args::parse();
	match args.command {
		Command::Init => init(&args).await,
		Command::Listen => listen(&args).await,
		Command::Connect(connect_args) => connect(&args, &connect_args).await,
		Command::List => list(&args).await,
		Command::Chat => chat(&args).await,
		Command::Completion(completion_args) => gen_completion(completion_args.shell),
	}
}

async fn init(args: &Args) {
	let conf = Conf::load(&args.conf_path).unwrap_or_else(|e| {
		error!("failed to load config: {e}");
		exit(1);
	});

	let peer_info = PeerInfo::new(conf.net.addr, conf.chat.addr, &conf.path.peer_info).await;
	if let Err(e) = peer_info.save().await {
		error!("failed to save peer info: {e}");
		exit(1);
	}

	let rsa = Rsa::generate(conf.crypto.rsa_bits).unwrap();

	let private_key = rsa.private_key_to_pem().unwrap();
	create_dir_all(conf.path.private_key.parent().unwrap()).await.unwrap();
	File::create(&conf.path.private_key).await.unwrap().write_all(&private_key).await.unwrap();

	let public_key = rsa.public_key_to_pem().unwrap();
	create_dir_all(conf.path.public_key.parent().unwrap()).await.unwrap();
	File::create(&conf.path.public_key).await.unwrap().write_all(&public_key).await.unwrap();
}

async fn listen(args: &Args) {
	let conf = Conf::load(&args.conf_path).unwrap_or_else(|e| {
		error!("failed to load config: {e}");
		exit(1);
	});
	let peer_info = PeerInfo::load(&conf.path.peer_info).await.unwrap_or_else(|e| {
		error!("failed to load peer info: {e}");
		exit(1);
	});
	rpc::server::listen(&peer_info).await;
}

async fn connect(args: &Args, connect_args: &ConnectArgs) {
	let conf = Conf::load(&args.conf_path).unwrap_or_else(|e| {
		error!("failed to load config: {e}");
		exit(1);
	});
	let mut peer_info = PeerInfo::load(&conf.path.peer_info).await.unwrap_or_else(|e| {
		error!("failed to load peer info: {e}");
		exit(1);
	});
	rpc::client::connect(connect_args.addr, &mut peer_info).await;
}

async fn list(args: &Args) {
	let conf = Conf::load(&args.conf_path).unwrap_or_else(|e| {
		error!("failed to load config: {e}");
		exit(1);
	});
	let peer_info = PeerInfo::load(&conf.path.peer_info).await.unwrap_or_else(|e| {
		error!("failed to load peer info: {e}");
		exit(1);
	});
	print_peers(peer_info.peers);
}

async fn chat(args: &Args) {
	let conf = Conf::load(&args.conf_path).unwrap_or_else(|e| {
		error!("failed to load config: {e}");
		exit(1);
	});
	let peer_info = PeerInfo::load(&conf.path.peer_info).await.unwrap_or_else(|e| {
		error!("failed to load peer info: {e}");
		exit(1);
	});
	rpc::chat::start(&peer_info).await;
}

fn print_peers(peers: HashMap<Uuid, Peer>) {
	println!("{:<38} {:<23} {:<20} {:<10}", "ID", "Address", "Last Seen", "Status");
	println!("{}", "-".repeat(100));

	for (id, peer) in peers {
		let time_ago = peer
			.last_seen
			.map(|l| format_duration_ago(l.elapsed().unwrap()))
			.unwrap_or("never".to_owned());
		println!("{:<38} {:<23} {:<20} {:<10}", id.to_string(), peer.addr, time_ago, peer.status);
	}
}

fn format_duration_ago(duration: Duration) -> String {
	let secs = duration.as_secs();
	match secs {
		0..=59 => format!("{secs} second(s) ago"),
		60..=3599 => format!("{} minute(s) ago", secs / 60),
		3600..=86399 => format!("{} hour(s) ago", secs / 3600),
		_ => format!("{} day(s) ago", secs / 86400),
	}
}
