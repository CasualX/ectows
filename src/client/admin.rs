use obfstr::obfstr;
use super::send;

const MAX_LOG_LINES: usize = 1000;

#[derive(Default)]
pub struct AdminClient {
	log_index: usize,
	pub(crate) has_ui: bool,
}

impl AdminClient {
	pub fn tick(&mut self, logs: &[String], tx: &mut websock::WebSocketTx) {
		// Catch out of sync issues with logs
		if self.log_index > logs.len() {
			self.log_index = logs.len();
		}
		// Drop logs if too far behind
		if logs.len() - self.log_index > MAX_LOG_LINES {
			self.log_index = logs.len() - MAX_LOG_LINES;
		}
		// Send any unsent logs
		while let Some(log) = logs.get(self.log_index) {
			self.log_index += 1;
			send(tx, obfstr!("console/log"), log);
		}
	}
}
