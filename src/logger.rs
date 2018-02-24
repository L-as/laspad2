pub trait Log {
	fn write(&self, priority: i64, line: &str);
}

macro_rules! elog {
	($log:expr, $priority:expr; $($arg:tt)*) => {{
		$crate::logger::Log::write($log, i64::max_value().overflowing_sub($priority).0, &format!($($arg)*));
	}};
	($log:expr; $($arg:tt)*) => {{
		elog!($log, 0; $($arg)*);
	}}
}

macro_rules! log {
	($log:expr, $verbosity:expr; $($arg:tt)*) => {
		elog!($log, i64::max_value().overflowing_add($verbosity).0; $($arg)*);
	};
	($log:expr; $($arg:tt)*) => {
		log!($log, 0; $($arg)*);
	}
}
