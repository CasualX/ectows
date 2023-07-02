use std::{io, net};
use std::collections::HashMap;
use obfstr::obfstr;
use super::*;

mod web;
mod admin;

pub use self::web::WebClient;
pub use self::admin::AdminClient;

pub enum TheClient {
	Web(WebClient),
	Admin(AdminClient),
}

pub struct Connection {
	pub stream: net::TcpStream,
	pub addr: net::SocketAddr,
	pub closing: bool,
	pub ws: WebSocket,
	pub cl: Option<TheClient>,
}

impl Connection {
	pub fn new(stream: net::TcpStream, addr: net::SocketAddr) -> Connection {
		Connection {
			stream, addr,
			closing: false,
			ws: WebSocket::server(websock::VecStrategy { capacity: 4000 }),
			cl: None,
		}
	}

	#[inline(never)]
	pub fn tick(&mut self, tokens: &Tokens, logs: &[String], tree: &mut dyn cvar::IVisit) {
		// Fill the recv buffer with data from the connection
		match self.ws.recv(&mut self.stream) {
			Ok(n) => {
				if n == 0 {
					self.closing = true;
				}
			}
			Err(err) => {
				if err.kind() != io::ErrorKind::WouldBlock {
					self.closing = true;
				}
			},
		}

		let mut handler = ConnHandler {
			// addr: &self.addr,
			closing: &mut self.closing,
			cl: &mut self.cl,
			tree,
			tokens,
		};

		// Parse incoming data and dispatch handlers
		match self.ws.dispatch(&mut handler) {
			Ok(()) => (),
			Err(_) => {
				self.closing = true;
			},
		}

		// Announce to web clients which game we're attached to
		if let Some(tx) = self.ws.transmit() {
			match self.cl {
				Some(TheClient::Web(ref mut web)) => web.tick(&self.addr, tx),
				Some(TheClient::Admin(ref mut admin)) => admin.tick(logs, tx),
				_ => (),
			}
		}

		// Send outgoing data
		match self.ws.send(&mut self.stream) {
			Ok(()) => (),
			Err(err) => if err.kind() != io::ErrorKind::WouldBlock {
				self.closing = true;
			},
		}
	}
}

struct ConnHandler<'a> {
	// addr: &'a net::SocketAddr,
	closing: &'a mut bool,
	cl: &'a mut Option<TheClient>,
	tree: &'a mut dyn cvar::IVisit,
	tokens: &'a Tokens,
}
impl<'a> websock::Handler for ConnHandler<'a> {
	fn http_request(&mut self, method: &str, uri: &str) -> websock::Result<()> {
		if method != obfstr!("GET") {
			return Err(websock::Error::METHOD_NOT_ALLOWED);
		}

		let pos = uri.find("?").unwrap_or(uri.len());
		let resource = uri.get(..pos).unwrap_or("");
		let query = uri.get(pos + 1..).unwrap_or("");

		let mut props = HashMap::new();
		load_props(&mut props, query);

		let token = props.get(obfstr!("token")).ok_or(websock::Error::UNAUTHORIZED)?.as_str();

		if resource == obfstr!("/") {
			if !self.tokens.is_web_token(token) {
				return Err(websock::Error::UNAUTHORIZED);
			}
			*self.cl = Some(TheClient::Web(WebClient::default()));
		}
		else if resource == obfstr!("/admin") {
			if !self.tokens.is_admin_token(token) {
				return Err(websock::Error::UNAUTHORIZED);
			}
			*self.cl = Some(TheClient::Admin(AdminClient::default()));
		}
		else {
			return Err(websock::Error::NOT_FOUND);
		}

		let visit = match self.cl {
			Some(TheClient::Web(web)) => {
				web as &mut dyn cvar::IVisit
			},
			Some(TheClient::Admin(_)) => {
				&mut *self.tree
			},
			None => return Ok(()),
		};

		cvar::console::walk(visit, |name, node| {
			if let Some(value) = props.get(name) {
				if let cvar::Node::Prop(prop) = node.as_node() {
					prop.set(value, &mut cvar::NullWriter);
				}
			}
		});

		Ok(())
	}

	fn message(&mut self, ws: &mut websock::WebSocketTx, msg: websock::Msg<'_>) {
		match msg {
			websock::Msg::Text(text) => {
				let tree = match self.cl {
					Some(TheClient::Web(web)) => {
						web as &mut dyn cvar::IVisit
					},
					Some(TheClient::Admin(_)) => {
						&mut *self.tree
					},
					None => return,
				};

				let mut response = String::new();
				let (path, args) = split_line(text);
				cvar::console::poke(tree, path, args, &mut response);

				// Reply with the log if any was written
				if response.len() > 0 {
					send(ws, obfstr!("console/log"), &response);
				}
			},
			websock::Msg::Binary(_) => (),
			websock::Msg::Close(close, reason) => {
				*self.closing = true;
				ws.send_close(close, reason);
			},
			websock::Msg::Ping(payload) => ws.send_pong(payload),
			websock::Msg::Pong(_) => (),
		}
	}
}

fn load_props(props: &mut HashMap<String, String>, args: &str) {
	for arg in args.split("&") {
		let key;
		let value;
		if let Some(pos) = arg.bytes().position(|b| b == b'=') {
			key = match arg.get(..pos) {
				Some(key) => key.to_owned(),
				_ => continue,
			};
			value = match arg.get(pos + 1..) {
				Some(value) => value,
				_ => continue,
			};
		}
		else {
			key = String::new();
			value = arg;
		}
		props.insert(key, value.to_owned());
	}
}

pub fn send<T: serde::Serialize>(ws: &mut websock::WebSocketTx, target: &str, message: T) {
	let text = Message { message, target }.to_string();
	ws.send_text(&text, true);
}
