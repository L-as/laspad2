use urlencoding;
use webview::{self, WebView};
use nfd::{self, Response};
use std::{thread, env, process::exit, time::Duration, path::{PathBuf, Path}};
use failure::*;

type Result<T> = ::std::result::Result<T, Error>;

pub fn init<T>(webview: &mut WebView<T>, _: &mut T) -> Result<()> {
	println!("init");
	webview.inject_css(CSS);
	webview.eval(ZEPTO);
	webview.eval(JS);
	fn get_file<T>(webview: &mut WebView<T>) -> Result<PathBuf> {
		match nfd::open_pick_folder(None)? {
			Response::Okay(dir) => Ok(PathBuf::from(dir)),
			Response::Cancel    => exit(1),
			_                   => {webview.eval("invalidproject()"); get_file(webview)}
		}
	};
	let dir = get_file(webview)?;
	env::set_current_dir(dir)?;
	if Path::new("laspad.toml").exists() {
		webview.eval("existingproject()");
	} else {
		webview.eval("newproject()");
	};
	Ok(())
}

pub fn main() -> Result<()> {
	let url = "data:text/html,".to_string() + &urlencoding::encode(HTML);
	let (_userdata, success) = webview::run("laspad", &url, Some((640, 640)), /*resizable*/ true, /*debug*/ true, |webview| {
		webview.dispatch(|webview, userdata| init(webview, userdata).unwrap());
		thread::sleep(Duration::from_millis(100));
	}, |webview, arg, userdata| {
		println!("callback: {}", arg);
		match arg {
			"exit" => exit(1),
			"init" => init(webview, userdata).unwrap(),
			_      => {},
		};
	}, 0);
	if !success {
		error!("Webview failed");
		exit(1)
	};

	Ok(())
}

const HTML:   &'static str = include_str!("ui.html");
const CSS:    &'static str = include_str!("ui.css");
const JS:     &'static str = include_str!("ui.js");
const ZEPTO:  &'static str = include_str!("../3rdparty/zepto.min.js");
