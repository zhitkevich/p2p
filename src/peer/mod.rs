use crate::crypto::Uuid;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::time::SystemTime;

pub mod info;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Peer {
	pub id: Uuid,
	pub addr: SocketAddr,
	pub chat_addr: SocketAddr,
	pub status: Status,
	pub last_seen: Option<SystemTime>,
}

impl Peer {
	pub fn new<I, A>(id: I, addr: A, chat_addr: A) -> Self
	where
		I: Into<Uuid>,
		A: Into<SocketAddr>,
	{
		Self {
			id: id.into(),
			addr: addr.into(),
			chat_addr: chat_addr.into(),
			status: Status::Offline,
			last_seen: None,
		}
	}
}

#[derive(
	Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize,
)]
pub enum Status {
	#[default]
	#[serde(rename = "online")]
	Online,
	#[serde(rename = "offline")]
	Offline,
}

impl Display for Status {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Online => write!(f, "online"),
			Self::Offline => write!(f, "offline"),
		}
	}
}
