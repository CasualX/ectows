use std::fmt;
use obfstr::obfstr;

/// The role associated with a token.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Role {
	User,
	Admin,
}
impl fmt::Display for Role {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Role::User => obfstr!("User").fmt(f),
			Role::Admin => obfstr!("Admin").fmt(f),
		}
	}
}
impl serde::Serialize for Role {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.collect_str(&self)
	}
}

#[derive(Default)]
pub struct Tokens {
	web: Vec<String>,
	admin: Vec<String>,
	insecure_check: usize,
}

impl Tokens {
	pub fn add(&mut self, role: Role, token: String) {
		let tokens = match role {
			Role::User => &mut self.web,
			Role::Admin => &mut self.admin,
		};
		tokens.push(token);
	}
	pub fn check_warn_insecure(&mut self) -> bool {
		let mut warn = false;
		if self.admin.len() != !self.insecure_check {
			self.insecure_check = !self.admin.len();
			warn = self.admin.is_empty();
		}
		warn
	}
	pub fn is_web_token(&self, token: &str) -> bool {
		self.web.binary_search_by_key(&token, |s| s.as_str()).is_ok()
	}
	pub fn is_admin_token(&self, token: &str) -> bool {
		// Use hardcoded admin token if there are no admin tokens
		if self.admin.is_empty() {
			return token == obfstr::obfstr!("admin");
		}
		self.admin.binary_search_by_key(&token, |s| s.as_str()).is_ok()
	}
}
