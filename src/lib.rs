#![feature(conservative_impl_trait)]
#![feature(proc_macro)]
#![feature(custom_attribute)]
#![allow(unused_attributes)]

#[macro_use]
extern crate telebot_derive;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;
extern crate curl;
extern crate futures;
extern crate tokio_core;
extern crate tokio_curl;

pub mod bot;
pub mod error;
pub mod objects;
pub mod functions;
