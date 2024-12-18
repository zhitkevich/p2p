use clap::{CommandFactory, ValueHint};
use clap_complete::{generate, Shell};
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(clap::Parser, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[command(version, about)]
pub struct Args {
	#[arg(
		short,
		long = "config",
		value_name = "PATH",
		value_hint = ValueHint::FilePath,
		default_value = "config.toml",
		global = true,
		help = "Config file path"
    )]
	pub conf_path: PathBuf,
	#[command(subcommand)]
	pub command: Command,
}

#[derive(clap::Subcommand, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Command {
	#[default]
	#[command(about = "Initializes files")]
	Init,
	#[command(about = "Listens for connections")]
	Listen,
	#[command(about = "Connects to a peer")]
	Connect(ConnectArgs),
	#[command(alias = "ls", about = "Lists connected peers")]
	List,
	#[command(about = "Starts realtime chat with connected peers")]
	Chat,
	#[command(about = "Generates shell completions")]
	Completion(CompletionArgs),
}

#[derive(clap::Args, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ConnectArgs {
	#[arg(value_name = "ADDRESS", value_hint = ValueHint::Hostname, help = "Peer address")]
	pub addr: SocketAddr,
}

#[derive(clap::Args, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CompletionArgs {
	#[arg(value_name = "SHELL", help = "Shell")]
	pub shell: Shell,
}

pub fn gen_completion(shell: Shell) {
	generate(shell, &mut Args::command(), env!("CARGO_BIN_NAME"), &mut io::stdout());
}
