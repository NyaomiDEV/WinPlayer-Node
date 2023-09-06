use napi_derive::napi;

mod player;
mod playermanager;
mod types;
mod util;

#[napi]
fn hello() -> String {
	String::from("Hello")
}