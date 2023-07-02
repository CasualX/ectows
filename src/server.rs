use std::{fmt, io, net};
use obfstr::obfstr;
use super::*;

/// The Ecto server.
///
/// The server is non-blocking, the [tick](Server::tick) function must be called at regular intervals (e.g. 60 times per second).
pub struct Server {
	listener: net::TcpListener,
	connections: Vec<Connection>,
	count: usize,
	tokens: Tokens,
	logs: Vec<String>,
}

impl Server {
	/// Creates a new ecto server listening on the given port.
	#[inline(never)]
	pub fn create(port: u16) -> io::Result<Server> {
		let sock_addr = net::SocketAddrV4::new(net::Ipv4Addr::UNSPECIFIED, port);
		let listener = net::TcpListener::bind(sock_addr)?;
		listener.set_nonblocking(true)?;
		Ok(Server {
			listener,
			connections: Vec::new(),
			count: 0,
			tokens: Tokens::default(),
			logs: Vec::new(),
		})
	}

	/// Adds a token to the server.
	///
	/// Clients connect with a token assigned a specific role.
	///
	/// If there are no admin tokens, a default admin token of 'admin' is accepted.
	pub fn add_token(&mut self, role: Role, token: String) {
		self.tokens.add(role, token);
	}

	/// Accepts incoming connections.
	#[inline(never)]
	fn accept(&mut self) {
		if self.tokens.check_warn_insecure() {
			self.log(fmtools::format_args!("WARNING: No admin tokens set, accepting default 'admin' token."));
		}
		while let Ok((stream, addr)) = self.listener.accept() {
			if let Ok(()) = stream.set_nonblocking(true) {
				self.log(fmtools::format_args!({addr}" connected"));
				let conn = Connection::new(stream, addr);
				self.connections.push(conn);
			}
			else {
				drop(stream);
			}
			self.count += 1;
		}
	}

	/// Do server things.
	///
	/// Accepts new connections, ticks existing connections, and cleans up closed connections.
	#[inline(never)]
	pub fn tick(&mut self, tree: &mut dyn cvar::IVisit) {
		self.accept();
		let logs = &self.logs;
		let tokens = &self.tokens;
		self.connections.retain_mut(move |conn| {
			conn.tick(tokens, logs, tree);
			!conn.closing
		});
	}

	/// Visualizes content.
	pub fn visualize(&mut self, scope: &str, content: fmt::Arguments) {
		let text = Message {
			message: Visualizer {
				scope,
				content,
			},
			target: obfstr!("debug/write"),
		}.to_string();

		for conn in self.connections.iter_mut() {
			if let Some(tx) = conn.ws.transmit() {
				if let Some(TheClient::Admin(_)) = conn.cl {
					tx.send_text(&text, true);
				}
			}
		}
	}

	/// Logs a message.
	pub fn log(&mut self, args: fmt::Arguments) {
		let mut line = String::new();
		_ = fmt::write(&mut line, args);
		if !line.ends_with("\n") {
			line.push_str("\n");
		}
		print!("{}", line);
		self.logs.push(line);
	}
}
