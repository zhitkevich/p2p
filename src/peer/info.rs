use crate::crypto::{Uuid, UuidV4};
use crate::peer::Peer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tokio::fs::read_to_string;
use tokio::{fs, io};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct PeerInfo {
	pub id: Uuid,
	pub addr: SocketAddr,
	pub chat_addr: SocketAddr,
	pub peers: HashMap<Uuid, Peer>,
	path: PathBuf,
}

impl PeerInfo {
	pub async fn new<A, P>(addr: A, chat_addr: A, path: P) -> Self
	where
		A: Into<SocketAddr>,
		P: AsRef<Path>,
	{
		Self {
			id: UuidV4::new().into(),
			addr: addr.into(),
			chat_addr: chat_addr.into(),
			peers: HashMap::new(),
			path: path.as_ref().to_path_buf(),
		}
	}

	/// Loads peer info from a file.
	///
	/// # Errors
	///
	/// If the file doesn't exist, error kind is [`ErrorKind::FileNotFound`].
	/// If there is an error while reading from the file, error kind is [`ErrorKind::ReadError`].
	/// If the file can't be parsed into peer info, error kind is [`ErrorKind::InvalidData`].
	pub async fn load<P>(path: P) -> Result<Self, Error>
	where
		P: AsRef<Path>,
	{
		serde_json::from_str::<Self>(&read_to_string(path).await.map_err(|e| match e.kind() {
			io::ErrorKind::NotFound => Error::new(ErrorKind::FileNotFound, "file not found"),
			_ => Error::new(ErrorKind::ReadError, e),
		})?)
		.map_err(|_| Error::new(ErrorKind::InvalidData, "file is malformed"))
	}

	/// Saves peer info to the file.
	///
	/// Recursively creates file if it doesn't exist.
	///
	/// # Errors
	///
	/// If peer info serialization fails, error kind is [`ErrorKind::InvalidData`].
	/// If there is an error while recursively creating the file or writing to it, error kind is
	/// [`ErrorKind::WriteError`].
	pub async fn save(&self) -> Result<(), Error> {
		if let Some(parent) = Path::new(&self.path).parent() {
			fs::create_dir_all(parent).await.map_err(|e| Error::new(ErrorKind::WriteError, e))?;
		}
		fs::write(
			&self.path,
			serde_json::to_vec(&self)
				.map_err(|_| Error::new(ErrorKind::InvalidData, "peer info is malformed"))?,
		)
		.await
		.map_err(|e| Error::new(ErrorKind::WriteError, e))
	}

	/// Retrieves an existing peer, or creates a new one if it doesn't exist.
	pub fn peer_or_insert<I, A>(
		&mut self,
		id: I,
		default_addr: A,
		default_chat_addr: A,
	) -> &mut Peer
	where
		I: Into<Uuid>,
		A: Into<SocketAddr>,
	{
		let id = id.into();
		self.peers.entry(id).or_insert(Peer::new(id, default_addr, default_chat_addr))
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
	WriteError,
	InvalidData,
}
