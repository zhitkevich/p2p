use crate::crypto::Uuid;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::ErrorKind::{ConnectionAborted, InvalidData};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub trait ReadRequest: AsyncReadExt + Unpin {
	async fn read_req(&mut self, cap: usize) -> io::Result<Request>;
}

impl<W> ReadRequest for W
where
	W: AsyncReadExt + Unpin,
{
	/// Reads a request into a buffer with the specified capacity.
	///
	/// If the request exceeds the buffer size it will be truncated, causing it to be malformed.
	///
	/// # Errors
	///
	/// This function returns [`io::Error`] if underlying implementation of [`Self::read`] fails.
	/// If this function reads 0 bytes, error kind is [`ConnectionAborted`].
	///
	/// # Examples
	///
	/// ```rust
	/// let stream = TcpStream::connect("192.168.0.1:7040");
	///
	/// let ping = match stream.read_req().await {
	///     Ok(Request::Ping(ping)) => ping,
	///     Ok(req) => panic!("unexpected request: {req:?}"),
	///     Err(e) if e.kind() == ConnectionAborted => panic!("connection aborted"),
	///     Err(e) => panic!("failed to read request: {e}"),
	/// };
	///
	/// println!("received ping: {ping:?}");
	/// ```
	async fn read_req(&mut self, cap: usize) -> io::Result<Request> {
		let mut buf = vec![0; cap];
		match self.read(&mut buf).await? {
			0 => Err(io::Error::new(ConnectionAborted, "connection aborted")),
			n => serde_json::from_slice(&buf[..n]).map_err(|e| io::Error::new(InvalidData, e)),
		}
	}
}

pub trait WriteRequest: AsyncWriteExt + Unpin {
	async fn write_req<R>(&mut self, req: R) -> io::Result<()>
	where
		R: Into<Request>;
}

impl<W> WriteRequest for W
where
	W: AsyncWriteExt + Unpin,
{
	async fn write_req<R>(&mut self, req: R) -> io::Result<()>
	where
		R: Into<Request>,
	{
		self.write_all(&serde_json::to_vec(&req.into())?).await
	}
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum Request {
	#[serde(rename = "ping")]
	Ping(Ping),
	#[serde(rename = "pong")]
	Pong(Pong),
	#[serde(rename = "message")]
	Message(Message),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Ping {
	pub peer_id: Uuid,
	pub peer_addr: SocketAddr,
	pub peer_chat_addr: SocketAddr,
}

impl Ping {
	pub fn new<I, A>(peer_id: I, peer_addr: A, peer_chat_addr: A) -> Self
	where
		I: Into<Uuid>,
		A: Into<SocketAddr>,
	{
		Self {
			peer_id: peer_id.into(),
			peer_addr: peer_addr.into(),
			peer_chat_addr: peer_chat_addr.into(),
		}
	}
}

impl From<Ping> for Request {
	fn from(ping: Ping) -> Self {
		Self::Ping(ping)
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Pong {
	pub peer_id: Uuid,
	pub peer_chat_addr: SocketAddr,
}

impl Pong {
	pub fn new<I, A>(peer_id: I, peer_chat_addr: A) -> Self
	where
		I: Into<Uuid>,
		A: Into<SocketAddr>,
	{
		Self { peer_id: peer_id.into(), peer_chat_addr: peer_chat_addr.into() }
	}
}

impl From<Pong> for Request {
	fn from(pong: Pong) -> Self {
		Self::Pong(pong)
	}
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Message {
	pub peer_id: Uuid,
	pub text: String,
}

impl Message {
	pub fn new<I, T>(peer_id: I, text: T) -> Self
	where
		I: Into<Uuid>,
		T: AsRef<str>,
	{
		Self { peer_id: peer_id.into(), text: text.as_ref().to_string() }
	}
}

impl From<Message> for Request {
	fn from(msg: Message) -> Self {
		Self::Message(msg)
	}
}
