/*!
EctoWS Networking System.
*/

fn unsafe_as_static_str(s: &str) -> &'static str {
	unsafe { &*(s as *const str) }
}

macro_rules! unsafe_obfstr {
	($string:literal) => {{
		crate::unsafe_as_static_str(obfstr::obfstr!($string))
	}};
}

mod server;
mod client;
mod message;
mod tokens;

type WebSocket = websock::WebSocket<websock::VecStrategy>;

pub use self::server::Server;
use self::client::*;
use self::message::*;
use self::tokens::*;
pub use self::tokens::Role;

fn split_line(line: &str) -> (&str, Option<&str>) {
	// Trim any leading whitespace
	let line = line.trim_start_matches(|c: char| c.is_ascii_whitespace());
	// Split the line into path and arguments
	let path = line.split_ascii_whitespace().next().unwrap_or("");
	// Trim the arguments and return None if empty
	let args = line.get(path.len()..)
		.map(|args| args.trim_matches(|c: char| c.is_ascii_whitespace()))
		.and_then(|args| if args.len() == 0 { None } else { Some(args) });
	(path, args)
}
