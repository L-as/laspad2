use web_view;
use std::{
	thread,
	time,
	env,
	fmt::Display,
	process::exit,
};
use failure::*;
use hyper::{self, server};
use futures::future::{self, Future};

use logger::{self, Log};
use config;

type Result<T> = ::std::result::Result<T, Error>;

fn get_branches() -> Result<String> {
	use std::slice::SliceConcatExt;

	Ok(config::get()?.branches()?.as_slice().join(""))
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

	fn dispatch<F: FnOnce() -> Result<()> + Send + 'static>(&self, command: F) -> String {
		thread::spawn(move || {
			UILog::push(match command() {
				Ok(())  => String::from("FIN"),
				Err(e)  => format!("ERR{}", e),
			});
		});
		String::new()
	}
}

impl server::Service for UI {
	type Request  = server::Request;
	type Response = server::Response;
	type Error    = hyper::Error;
	type Future   = Box<Future<Item=Self::Response, Error=Self::Error>>;

	fn call(&self, req: Self::Request) -> Self::Future {
		use hyper::{Method, header::*};
		use mime;

		let resp = match (req.method(), req.path(), req.query()) {
			(&Method::Get, path, None) => {
				let (body, mime) = match path {
					"/"             => (HTML.as_str(), mime::TEXT_HTML_UTF_8),
					_               => {eprintln!("Invalid GET: {}", path); ("", mime::TEXT_PLAIN)},
				};
				Self::Response::new()
					.with_header(ContentLength(body.len() as u64))
					.with_header(Link::new(vec![LinkValue::new(path.to_owned()).set_media_type(mime)]))
					.with_body(body)
			},
			(&Method::Post, command, query) => { // returns String instead of &'static str
				let body = match (command, query) {
					("/exit",           None)         => exit(0),
					("/create_project", None)         => self.dispatch(|| ::init::main(false)),
					("/update",         None)         => self.dispatch(::update::main),
					("/publish",        Some(branch)) => {let branch = branch.to_owned(); self.dispatch(move || ::publish::main(&branch, false))},
					("/need",           Some(modid))  => {let modid  = modid .to_owned(); self.dispatch(move || ::need::main(&modid))},
					("/get_branches",   None)         => self.run(get_branches),
					("/get_msg",        None)         => UILog::remove(),
					_                                 => {eprintln!("Invalid POST: {}, {:?}", command, query); String::from("ERRSomething went wrong, please retry")},
				};
				Self::Response::new()
					.with_header(ContentLength(body.len() as u64))
					.with_body(body)
			},
			(method, path, query) => {eprintln!("Invalid request: {}, {}, {:?}", method, path, query); Self::Response::new()},
		};

		Box::new(future::ok(resp))
	}
}

pub fn main() -> Result<()> {
	fn sleep() {
		thread::sleep(time::Duration::from_millis(100)); // to avoid segfault
	}

	fn spawn_server() -> Result<()> {
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
	thread::spawn(move || if let Err(e) = spawn_server() {eprintln!("Server failed: {}", e); exit(1)});

	let (_, success) = web_view::run("laspad", "", Some((1600, 900)), true, true, move |webview| {
		use web_view::*;

		thread::spawn(move || {
			webview.dispatch(|webview, _| {
				sleep();
				let path = webview.dialog(DialogType::Open, DialogFlags::Directory, "Choose laspad project folder", None);
				if path.len() == 0 {
					exit(0);
				}
				env::set_current_dir(path).unwrap();
				webview.eval(r#"open('http://127.0.0.1:51823/', '_self', false)"#);
				sleep();
			});
			sleep();
		});
	}, |_, _, _| {sleep()}, ());

	if !success {
		eprintln!("Failed to execute webview");
		exit(1);
	}

	Ok(())
}

lazy_static! {
	static ref HTML: String = format!(
		include_str!("ui.html"),
		stylesheet = include_str!("ui.css"),
		javascript = include_str!("ui.js"),
	);
}
