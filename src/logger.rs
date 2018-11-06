use downcast::Any;
use std::{
	ops::{Deref, DerefMut},
	sync::{Mutex, MutexGuard},
};

pub struct State {
	log:          Option<Box<dyn Log>>,
	min_priority: i64,
}

impl Default for State {
	fn default() -> Self {
		State {
			log:          None,
			min_priority: -1,
		}
	}
}

impl Deref for State {
	type Target = Option<Box<dyn Log>>;

	fn deref(&self) -> &Self::Target {
		&self.log
	}
}

impl DerefMut for State {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.log
	}
}

lazy_static! {
	static ref MUTEX: Mutex<State> = Mutex::new(State::default());
}

pub trait Log: Send + Any {
	fn write(&mut self, priority: i64, line: &str);
}

downcast!(dyn Log);

pub fn set_priority(priority: i64) {
	let mut lock = MUTEX.lock().unwrap();
	lock.min_priority = priority;
}

pub fn get_priority() -> i64 {
	let lock = MUTEX.lock().unwrap();
	lock.min_priority
}

pub fn set(log: Box<dyn Log>) {
	let mut lock = MUTEX.lock().unwrap();
	lock.log = Some(log);
}

pub fn get() -> MutexGuard<'static, State> {
	MUTEX.lock().unwrap()
}

pub fn remove() -> Option<Box<dyn Log>> {
	let mut lock = MUTEX.lock().unwrap();
	lock.log.take()
}

pub fn log(priority: i64, line: &str) {
	let mut lock = MUTEX.lock().unwrap();
	if priority < lock.min_priority {
		return;
	};
	if let Some(ref mut log) = lock.log {
		log.write(priority, line);
	};
}

macro_rules! elog {
	($priority:expr; $($arg:tt)*) => {{
		let priority = i64::max_value().overflowing_sub($priority).0;
		if priority >= $crate::logger::get_priority() {
			$crate::logger::log(i64::max_value().overflowing_sub($priority).0, &format!($($arg)*));
		};
	}};
	($($arg:tt)*) => {{
		elog!(0; $($arg)*);
	}}
}

macro_rules! log {
	($verbosity:expr; $($arg:tt)*) => {
		elog!(i64::max_value().overflowing_add($verbosity).0; $($arg)*);
	};
	($($arg:tt)*) => {
		log!(0; $($arg)*);
	}
}
