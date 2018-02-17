use urlencoding;
use webview::{self, WebView};
use nfd::{self, Response};
use std::{thread, env, process::exit, time::Duration, path::PathBuf, ffi::OsStr};

pub fn main() {
	let url = "data:text/html,".to_string() + &urlencoding::encode(HTML);
	let (_userdata, success) = webview::run("laspad", &url, Some((640, 640)), /*resizable*/ true, /*debug*/ true, move |webview| {
		println!("Setup");
		thread::spawn(move || {
			webview.dispatch(|webview, _| {
				webview.inject_css(CSS);
				webview.eval(JS);
				fn get_file<T>(webview: &mut WebView<T>) -> PathBuf {
					let file = match nfd::open_file_dialog(None, None).unwrap() {
						Response::Okay(file) => PathBuf::from(file),
						Response::Cancel     => exit(1),
						_                    => {webview.eval("nofile()"); get_file(webview)}
					};
					if file.file_name() == Some(OsStr::new("laspad.toml")) && file.is_file() {
						file
					} else {
						webview.eval("nofile()");
						get_file(webview)
					}
				};
				let file = get_file(webview);
				env::set_current_dir(file.parent().unwrap()).unwrap();
			});
			thread::sleep(Duration::from_millis(100));
		});
	}, move |webview, arg, _| {
		println!("Callback");
	}, 0);
	if !success {
		error!("Webview failed");
		exit(1)
	};
}

const HTML: &'static str = include_str!("ui.html");
const CSS:  &'static str = include_str!("ui.css");
const JS:   &'static str = include_str!("ui.js");
