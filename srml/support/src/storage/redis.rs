use std::error::Error;
use std::string::ToString;

use parking_lot::Mutex;
use redis::{Connection, RedisError};
use redis::Commands;

use lazy_static::lazy_static;
use log::error;

struct RedisClient {
	conn: Option<Connection>
}

impl RedisClient {
	pub fn set_conn(&mut self, conn: Connection) {
		self.conn = Some(conn)
	}
	pub fn set_with_blocknumber(&self, key: &[u8], h: u64, value: &[u8]) {
		match self.conn {
			Some(ref conn) => {
				// remove current score first
				// let _: Result<usize, RedisError> = redis::cmd("zremrangebyscore").arg(key).arg(h).arg(h).query(conn);
				// let num = h.to_string();
				// let mut redis_key = key.to_vec();
				// redis_key.extend(b"#".to_vec());
				// redis_key.extend(num.as_bytes());
				// let _: Result<usize, RedisError> = redis::cmd("zadd").arg(key).arg(h).arg(redis_key.as_slice()).query(conn).map_err(|e| {
				// 	error!("[redis] [zadd] failed for {:}, current key:{:?}, value:{:?}", e, key, redis_key);
				// 	e
				// });
				// self.set(redis_key.as_slice(), value)
				let num = h.to_string();
				let mut redis_key = key.to_vec();
				redis_key.extend(b"#".to_vec());
				redis_key.extend(num.as_bytes());
				// use pipe
				let r: Result<(usize,), RedisError> = redis::pipe()
					.cmd("ZREMRANGEBYSCORE").arg(key).arg(h).arg(h).ignore()
					.cmd("ZADD").arg(key).arg(h).arg(redis_key.as_slice())
					.cmd("SET").arg(redis_key).arg(value).ignore()
					.query(conn);

				if let Err(e) = r {
					println!("redis err:{}", e);
				}
			}
			None => { return; }
		};
	}
	pub fn set(&self, key: &[u8], value: &[u8]) {
		let _: Result<String, RedisError> = match self.conn {
			Some(ref conn) => {
				conn.set(key, value)
			}
			None => { return; }
		}.map_err(|e| {
			error!("[redis] [set] failed for {:}, current key:{:?}, value:{:?}", e, key, value);
			e
		});
	}
	pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		match self.conn {
			Some(ref conn) => {
				conn.get(key)
			}
			None => { return None; }
		}.map_err(|e| {
			error!("[redis] [get] failed for {:}, current key:{:?}", e, key);
			e
		}).ok()
	}
}

lazy_static! {
	static ref REDIS: Mutex<RedisClient> = Mutex::new(RedisClient { conn: None });
}

pub fn init_redis(url: &str) -> Result<(), String> {
	let client = redis::Client::open(url).map_err(|e| e.description().to_string())?;
	let conn = client.get_connection().map_err(|e| e.description().to_string())?;
	REDIS.lock().set_conn(conn);
	Ok(())
}

pub fn redis_get(key: &[u8]) -> Option<Vec<u8>> {
	REDIS.lock().get(key)
}

pub fn redis_set(key: &[u8], value: &[u8]) {
	REDIS.lock().set(key, value)
}

pub fn redis_set_with_blocknumer(key: &[u8], h: u64, value: &[u8]) {
	REDIS.lock().set_with_blocknumber(key, h, value)
}
