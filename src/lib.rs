use napi_derive::napi;

mod player;
mod playermanager;
mod types;
mod util;

#[napi]
pub fn hello() -> String {
	String::from("Hello")
}