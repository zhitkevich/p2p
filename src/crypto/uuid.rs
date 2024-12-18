use rand::random;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Uuid {
	V4(UuidV4),
}

impl Default for Uuid {
	fn default() -> Self {
		Self::V4(UuidV4::default())
	}
}

impl Display for Uuid {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::V4(v4) => Display::fmt(v4, f),
		}
	}
}

impl Serialize for Uuid {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match self {
			Self::V4(v4) => serializer.serialize_str(&v4.to_string()),
		}
	}
}

impl<'de> Deserialize<'de> for Uuid {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		UuidV4::try_from(String::deserialize(deserializer)?)
			.map_err(de::Error::custom)
			.map(UuidV4::into)
	}
}

impl From<UuidV4> for Uuid {
	fn from(v4: UuidV4) -> Self {
		Self::V4(v4)
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Serialize, Deserialize)]
pub struct UuidV4([u8; 16]);

impl UuidV4 {
	pub fn new() -> Self {
		let mut rng: [u8; 16] = random();
		rng[6] = (rng[6] & 0x0f) | 0x40;
		rng[8] = (rng[8] & 0x3f) | 0x80;
		Self(rng)
	}
}

impl Display for UuidV4 {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let b = self.0;
		write!(
			f,
			"{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
			u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
			u16::from_le_bytes([b[4], b[5]]),
			u16::from_le_bytes([b[6], b[7]]),
			u16::from_le_bytes([b[8], b[9]]),
			u64::from_le_bytes([b[10], b[11], b[12], b[13], b[14], b[15], 0, 0])
		)
	}
}

impl Debug for UuidV4 {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(self, f)
	}
}

impl TryFrom<String> for UuidV4 {
	type Error = Error;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		let segments: Vec<_> = s.split('-').collect();
		if segments.len() != 5 {
			return Err(Error::new(ErrorKind::ParseError, "expected 5 hyphen-separated segments"));
		}
		let mut bytes = [0; 16];
		bytes[0..4].copy_from_slice(
			&u32::from_str_radix(segments[0], 16)
				.map_err(|_| Error::new(ErrorKind::ParseError, "found invalid hexadecimal"))?
				.to_le_bytes(),
		);
		bytes[4..6].copy_from_slice(
			&u16::from_str_radix(segments[1], 16)
				.map_err(|_| Error::new(ErrorKind::ParseError, "found invalid hexadecimal"))?
				.to_le_bytes(),
		);
		bytes[6..8].copy_from_slice(
			&u16::from_str_radix(segments[2], 16)
				.map_err(|_| Error::new(ErrorKind::ParseError, "found invalid hexadecimal"))?
				.to_le_bytes(),
		);
		bytes[8..10].copy_from_slice(
			&u16::from_str_radix(segments[3], 16)
				.map_err(|_| Error::new(ErrorKind::ParseError, "found invalid hexadecimal"))?
				.to_le_bytes(),
		);
		bytes[10..16].copy_from_slice(
			&u64::from_str_radix(segments[4], 16)
				.map_err(|_| Error::new(ErrorKind::ParseError, "found invalid hexadecimal"))?
				.to_le_bytes()[0..6],
		);
		Ok(Self(bytes))
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
	ParseError,
}
