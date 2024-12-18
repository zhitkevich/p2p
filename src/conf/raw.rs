use serde::Deserialize;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Conf {
	pub path: path::Conf,
	pub network: network::Conf,
	pub crypto: crypto::Conf,
	pub chat: chat::Conf,
}

pub mod path {
	use serde::Deserialize;

	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
	pub struct Conf {
		pub app: String,
		pub private_key: String,
		pub public_key: String,
		pub peer_info: String,
	}
}

pub mod network {
	use serde::Deserialize;
	use std::net::SocketAddr;

	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
	pub struct Conf {
		pub address: SocketAddr,
	}
}

pub mod crypto {
	use serde::Deserialize;

	#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
	pub struct Conf {
		pub rsa_bits: u32,
	}
}

pub mod chat {
	use serde::Deserialize;
	use std::net::SocketAddr;

	#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
	pub struct Conf {
		pub address: SocketAddr,
	}
}
