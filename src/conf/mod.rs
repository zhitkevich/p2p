use std::cmp::PartialEq;
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
use std::{env, fmt, fs};
use tokio::io;

mod raw;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Conf {
	pub path: path::Conf,
	pub net: net::Conf,
	pub crypto: crypto::Conf,
	pub chat: chat::Conf,
}

impl Conf {
	/// Loads config from a file.
	///
	/// # Errors
	///
	/// If the file doesn't exist, error kind is [`ErrorKind::FileNotFound`].
	/// If there is an error while reading from the file, error kind is [`ErrorKind::ReadError`].
	/// If the file can't be parsed into config, error kind is [`ErrorKind::InvalidData`].
	/// If the home environment variable is not set, error kind is [`ErrorKind::HomeNotFound`].
	pub fn load<P>(path: P) -> Result<Self, Error>
	where
		P: AsRef<Path>,
	{
		let raw_conf: raw::Conf =
			toml::from_str(&fs::read_to_string(path).map_err(|e| match e.kind() {
				io::ErrorKind::NotFound => Error::new(ErrorKind::FileNotFound, "file not found"),
				_ => Error::new(ErrorKind::ReadError, e),
			})?)
			.map_err(|_| Error::new(ErrorKind::InvalidData, "file is malformed"))?;

		let home = if cfg!(windows) {
			env::var("USERPROFILE").map_err(|e| Error::new(ErrorKind::HomeNotFound, e))?
		} else {
			env::var("HOME").map_err(|e| Error::new(ErrorKind::HomeNotFound, e))?
		};
		let app = PathBuf::from(&home).join(&raw_conf.path.app);
		let private_key = app.join(&raw_conf.path.private_key);
		let public_key = app.join(&raw_conf.path.public_key);
		let peers = app.join(&raw_conf.path.peer_info);

		Ok(Self {
			path: path::Conf { app, private_key, public_key, peer_info: peers },
			net: net::Conf { addr: raw_conf.network.address },
			crypto: crypto::Conf { rsa_bits: raw_conf.crypto.rsa_bits },
			chat: chat::Conf { addr: raw_conf.chat.address },
		})
	}
}

pub mod path {
	use std::path::PathBuf;

	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
	pub struct Conf {
		pub app: PathBuf,
		pub private_key: PathBuf,
		pub public_key: PathBuf,
		pub peer_info: PathBuf,
	}
}

pub mod net {
	use std::net::SocketAddr;

	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
	pub struct Conf {
		pub addr: SocketAddr,
	}
}

pub mod crypto {
	#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
	pub struct Conf {
		pub rsa_bits: u32,
	}
}

pub mod chat {
	use std::net::SocketAddr;

	#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
	pub struct Conf {
		pub addr: SocketAddr,
	}
}

#[derive(Debug)]
pub struct Error {
	pub kind: ErrorKind,
	pub err: Box<dyn std::error::Error + Send + Sync>,
}

impl Error {
	pub fn new<E>(kind: ErrorKind, err: E) -> Self
	where
		E: Into<Box<dyn std::error::Error + Send + Sync>>,
	{
		Self { kind, err: err.into() }
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.err)
	}
}

impl std::error::Error for Error {}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum ErrorKind {
	#[default]
	FileNotFound,
	ReadError,
	InvalidData,
	HomeNotFound,
}
