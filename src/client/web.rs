use std::net;
use obfstr::obfstr;
use crate::send;

#[derive(Default)]
pub struct WebClient {
	pub update: bool,
	pub login: bool,
	// pub game: Option<GameID>,
}

impl cvar::IVisit for WebClient {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		f(&mut cvar::Action(obfstr!("net.state!"), |_, _| self.update = true));
		f(&mut cvar::Action(obfstr!("net.login!"), |_, _| self.login = true));
	}
}

impl WebClient {
	pub fn tick(&mut self, addr: &net::SocketAddr, tx: &mut websock::WebSocketTx) {
		if self.login {
			self.login = false;
			send(tx, obfstr!("auth/welcome"), crate::Welcome {
				addr: *addr,
				role: crate::Role::User,
				name: obfstr!("Anonymous"),
			});
		}
		// if game != self.game {
		// 	if let Some(ref name) = self.game {
		// 		send(tx, obfstr!("detachGame"), name);
		// 	}
		// 	if let Some(name) = game {
		// 		send(tx, obfstr!("attachGame"), name);
		// 	}
		// 	self.game = game.clone();
		// }
	}
}
