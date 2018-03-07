//use webview::{self, WebView};
use nfd;
use toml;
use std::{
	thread,
	time,
	env,
	fs,
	process::exit,
	//time::Duration,
	path::{PathBuf, Path},
	fmt::Display,
};
use failure::*;
use hyper::{self, server};
use futures::future::{self, Future};

use logger::{self, Log};

type Result<T> = ::std::result::Result<T, Error>;

fn find_project() -> Result<&'static str> {
	fn get_file() -> Result<PathBuf> {
		match nfd::open_pick_folder(None)? {
			nfd::Response::Okay(dir) => Ok(PathBuf::from(dir)),
			nfd::Response::Cancel    => exit(1),
			_                        => {get_file()}
		}
	};
	let dir = get_file()?;
	env::set_current_dir(dir)?;
	if Path::new("laspad.toml").exists() {
		Ok("old")
	} else {
		Ok("new")
	}
}

fn get_branches() -> Result<String> {
	use std::slice::SliceConcatExt;

	let toml: toml::Value = fs::read_string("laspad.toml")?.parse()?;
	Ok(toml.as_table().ok_or_else(|| format_err!("laspad.toml is incorrectly formatted"))?.keys().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice().join(""))
}

struct UI;

struct UILog {
	sublog: Option<Box<Log>>,
	queue:  Vec<String>,
}

impl UILog {
	fn remove() -> String {
		{
			use std::ops::DerefMut;

			let mut log             = logger::get();
			let     log             = log.deref_mut().deref_mut();
			let     log             = log.as_mut().unwrap();
			let     log: &mut Log   = log.deref_mut();
			let     log: &mut UILog = log.downcast_mut().unwrap();

			if log.queue.len() > 0 {
				Some(log.queue.remove(0))
			} else {
				None
			}
		}.unwrap_or({
			thread::sleep(time::Duration::from_millis(100));
			String::new()
		})
	}

	fn push(s: String) {
		use std::ops::DerefMut;

		let mut log             = logger::get();
		let     log             = log.deref_mut().deref_mut();
		let     log             = log.as_mut().unwrap();
		let     log: &mut Log   = log.deref_mut();
		let     log: &mut UILog = log.downcast_mut().unwrap();

		log.queue.push(s);
	}
}

impl Log for UILog {
	fn write(&mut self, p: i64, line: &str) {
		let str = format!("{}{}", if p > 0 {"WRN"} else if p == 0 {"INF"} else {"LOG"}, line);
		self.queue.push(str);
		if let Some(ref mut sublog) = self.sublog {
			sublog.write(p, line);
		};
	}
}

impl UI {
	fn run<F: FnOnce() -> Result<T>, T: Display>(&self, f: F) -> String {
		match f() {
			Ok(res) => format!("FIN{}", res),
			Err(e)  => format!("ERR{}", e),
		}
	}

	fn dispatch<F: FnOnce() -> Result<()> + Send + 'static>(&self, command: F) -> &'static str {
		thread::spawn(move || {
			UILog::push(match command() {
				Ok(())  => String::from("FIN"),
				Err(e)  => format!("ERR{}", e),
			});
		});
		""
	}
}

impl server::Service for UI {
	type Request  = server::Request;
	type Response = server::Response;
	type Error    = hyper::Error;
	type Future   = Box<Future<Item=Self::Response, Error=Self::Error>>;

	fn call(&self, req: Self::Request) -> Self::Future {
		use hyper::{Method, header};

		let body = match (req.method(), req.path(), req.query()) {
			(&Method::Get,  "/",               None)         => HTML,
			(&Method::Get,  "/index.css",      None)         => CSS,
			(&Method::Get,  "/laspad-ui.js",   None)         => JS,
			(&Method::Post, "/create_project", None)         => self.dispatch(::init::main),
			(&Method::Post, "/update",         None)         => self.dispatch(::update::main),
			(&Method::Post, "/publish",        Some(branch)) => {let branch = branch.to_owned(); self.dispatch(move || ::publish::main(&branch, false))},
			(&Method::Post, "/need",           Some(modid))  => {let modid  = modid .to_owned(); self.dispatch(move || ::need::main(&modid))},
			(&Method::Post, command, query) => { // returns String instead of &'static str
				let body = match command {
					"/find_project" => self.run(find_project),
					"/get_branches" => self.run(get_branches),
					"/get_msg"      => UILog::remove(),
					_               => {eprintln!("Invalid POST: {}, {:?}", command, query); String::from("ERRSomething went wrong, please retry")},
				};
				return Box::new(future::ok(Self::Response::new()
					.with_header(header::ContentLength(body.len() as u64))
					.with_body(body)
				));
			}
			_ => {eprintln!("Invalid GET!"); "FIN"},
		};

		Box::new(future::ok(Self::Response::new()
			.with_header(header::ContentLength(body.len() as u64))
			.with_body(body)
		))
	}
}

pub fn main() -> Result<()> {
	let addr = "127.0.0.1:51823".parse()?;
	let server = server::Http::new().bind(&addr, move || Ok(UI))?;
	let log = UILog {
		sublog: logger::remove(),
		queue:  Vec::new(),
	};
	logger::set(Box::new(log));
	server.run()?;
	Ok(())
}

static HTML: &'static str  = include_str!("ui.html");
static CSS:  &'static str  = include_str!("ui.css");
static JS:   &'static str  = include_str!("ui.js");
