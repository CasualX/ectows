use std::{fmt, net};
use crate::Role;

//----------------------------------------------------------------

/// Welcome message.
///
/// Replies with the user's IP address, role and their name.
#[derive(Clone, Debug)]
pub struct Welcome<'a> {
	pub addr: net::SocketAddr,
	pub role: Role,
	pub name: &'a str,
}
impl serde::Serialize for Welcome<'_> {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		use serde::ser::*;
		let mut state = serializer.serialize_struct(unsafe_obfstr!("Welcome"), 3)?;
		state.serialize_field(unsafe_obfstr!("addr"), &self.addr)?;
		state.serialize_field(unsafe_obfstr!("role"), &self.role)?;
		state.serialize_field(unsafe_obfstr!("name"), &self.name)?;
		state.end()
	}
}

//----------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Visualizer<'a> {
	pub scope: &'a str,
	pub content: fmt::Arguments<'a>,
}
impl<'a> serde::Serialize for Visualizer<'a> {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		use serde::ser::*;
		let mut state = serializer.serialize_struct(unsafe_obfstr!("Visualizer"), 2)?;
		state.serialize_field(unsafe_obfstr!("scope"), &self.scope)?;
		state.serialize_field(unsafe_obfstr!("content"), &self.content)?;
		state.end()
	}
}

//----------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct StringTable<'a> {
	pub name: &'a str,
	pub values: &'a [&'a str],
}
impl<'a> serde::Serialize for StringTable<'a> {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		use serde::ser::*;
		let mut state = serializer.serialize_struct(unsafe_obfstr!("StringTable"), 2)?;
		state.serialize_field(unsafe_obfstr!("name"), &self.name)?;
		state.serialize_field(unsafe_obfstr!("values"), &self.values)?;
		state.end()
	}
}

//----------------------------------------------------------------

/// Messages sent to the clients.
///
/// Messages have content and target a module.
#[derive(Clone, Debug)]
pub struct Message<'a, T> {
	pub message: T,
	pub target: &'a str,
}
impl<T: serde::Serialize> serde::Serialize for Message<'_, T> {
	#[inline(never)]
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		use serde::ser::*;
		let mut state = serializer.serialize_struct(unsafe_obfstr!("Message"), 2)?;
		state.serialize_field(unsafe_obfstr!("message"), &self.message)?;
		state.serialize_field(unsafe_obfstr!("target"), &self.target)?;
		state.end()
	}
}
impl<T: serde::Serialize> fmt::Display for Message<'_, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match serde_json::to_string(self) {
			Ok(s) => s.fmt(f),
			Err(_) => Err(fmt::Error),
		}
	}
}
