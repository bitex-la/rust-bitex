#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate curs;
extern crate mime_guess;
extern crate serde;

use std::fs::File;
use std::path::Path;
use curs::hyper::client::{Client, IntoUrl};
pub use curs::hyper::client::response::Response;
use curs::hyper::error::Error as HyperError;
pub use curs::hyper::method::Method;
use std::io::Error as IoError;
use std::io::Read;
use std::fmt;
use std::fmt::Write;
use curs::hyper::mime::{Mime, TopLevel, SubLevel};
use curs::{CursResult, Request, DecodableResult, Param, Params};
use curs::serde::{Deserialize, Deserializer};
use curs::serde::de::impls::TupleVisitor12;

pub use curs::hyper::status::StatusCode;

#[doc(no_inline)]
pub use curs::hyper::header::{Headers, Header, HeaderFormat, UserAgent, ContentType};

#[derive(Debug, Clone)]
pub struct Api<'a> {
  key: &'a str,
  url_base: &'a str
}

type OrderVisitor =
  TupleVisitor12<i64, i64, i64, i64, f64, f64, f64, i64, i64, f64, Option<String>, f64>;

type OrderTuple =
  (i64, i64, i64, i64, f64, f64, f64, i64, i64, f64, Option<String>, f64);

macro_rules! make_order_endpoint {
  ($name:ident, $api:ident, $order_type_value:expr, $endpoint_name:expr) => (
    #[derive(Debug)]
    pub struct $api<'a> {
      api: Api<'a>
    }

    impl <'a> $api<'a> {
      pub fn show(&self, id: i64) -> CursResult<$name> {
        self.api.private_get(&["private/", $endpoint_name, "/", &*(id.to_string())].concat(), vec![])
      }

      pub fn create(&self, amount: f64, price: f64) -> CursResult<$name> {
        self.api.private_post("private/bids",
          vec![("amount", &*amount.to_string()), ("price", &*price.to_string())])
      }
    }

    #[derive(Debug)]
    pub struct $name {
      pub id: i64,
      pub creation: i64,
      pub orderbook: i64,
      pub amount_to_spend: f64,
      pub remaining_amount: f64,
      pub price: f64,
      pub status: i64,
      pub cancelation_reason: i64,
      pub produced_amount: f64,
      pub issuer: Option<String>,
      pub fees_paid: f64
    }

    impl $name {
      fn from_tuple(t: OrderTuple) -> $name {
        $name {
          id: t.1,
          creation: t.2,
          orderbook: t.3,
          amount_to_spend: t.4,
          remaining_amount: t.5,
          price: t.6,
          status: t.7,
          cancelation_reason: t.8,
          produced_amount: t.9,
          issuer: t.10,
          fees_paid: t.11
        }
      }
    }

    impl Deserialize for $name {
      fn deserialize<D: Deserializer>(d: &mut D) -> Result<$name, D::Error>{
        let tuple = try!(d.deserialize(OrderVisitor::new()));
        if tuple.0 == $order_type_value {
          Ok($name::from_tuple(tuple))
        }else{
          panic!("Unexpected order type")
        }
      }
    }
  )
}

make_order_endpoint!{ Bid, BidsApi, 1, "bids" }
make_order_endpoint!{ Ask, AsksApi, 2, "asks" }

#[derive(Debug)]
pub enum Order {
  Bid(Bid),
  Ask(Ask)
}

impl Deserialize for Order {
  fn deserialize<D: Deserializer>(d: &mut D) -> Result<Order, D::Error>{
    let tuple = try!(d.deserialize(OrderVisitor::new()));
    if tuple.0 == 1 {
      Ok(Order::Bid(Bid::from_tuple(tuple)))
    }else{
      Ok(Order::Ask(Ask::from_tuple(tuple)))
    }
  }
}

#[derive(Deserialize, Debug)]
pub struct OrderBook {
  pub bids: Vec<(f64, f64)>,
  pub asks: Vec<(f64, f64)>
}

#[derive(Deserialize, Debug)]
pub struct Profile {
  pub usd_balance: f64,
  pub usd_reserved: f64,
  pub usd_available: f64,
  pub btc_balance: f64,
  pub btc_reserved: f64,
  pub btc_available: f64,
  pub fee: f64,
  pub btc_deposit_address: String,
  pub more_mt_deposit_code: String
}

pub const PRODUCTION_URL_BASE: &'static str = "https://bitex.la";
pub const SANDBOX_URL_BASE: &'static str = "https://sandbox.bitex.la";

impl<'a> Api<'a> {
  /// Creates a new client pointing to the given URL.
  /// Bitex.la production and sandbox urls are exported
  /// as constants here, but you may use a different one
  /// when testing. Checkout the prod and sandbox shortcuts too.
  pub fn new(url: &'a str) -> Api<'a>{
    Api{ key: "", url_base: url}
  }

  /// Shortcut if you just want a production Api client.
  pub fn prod() -> Api<'a>{
    Api{ key: "", url_base: PRODUCTION_URL_BASE}
  }

  /// Shortcut if you just want a sandbox Api client.
  pub fn sandbox() -> Api<'a>{
    Api{ key: "", url_base: PRODUCTION_URL_BASE}
  }

  // Set the current API key for authenticated requests.
  pub fn key(mut self, key: &'a str) -> Api {
    self.key = key;
    self
  }

  fn url(&self, endpoint: &str) -> String {
    [self.url_base, "/api-v1/rest/", endpoint].concat()
  }

  fn add_key<'b>(&'b self, params: Params<'b>) -> Params<'b> {
    let mut with_key = vec![("api_key", &*self.key)];
    with_key.extend(params);
    with_key
  }

  pub fn post<'b, D>(&self, endpoint: &str, params: Params<'b>) -> CursResult<D>
    where D: Deserialize
  {
    curs::Request::new(curs::Method::Post, &self.url(endpoint))
      .params(params).send().decode_success()
  }

  pub fn get<'b, D>(&self, endpoint: &str, params: Params<'b>) -> CursResult<D>
    where D: Deserialize
  {
    curs::Request::new(curs::Method::Get, &self.url(endpoint))
      .params(params).send().decode_success()
  }

  pub fn private_post<'b, D>(&'b self, endpoint: &str, params: Params<'b>) -> CursResult<D>
    where D: Deserialize
  {
    self.post(endpoint, self.add_key(params))
  }

  pub fn private_get<'b, D>(&'b self, endpoint: &str, params: Params<'b>) -> CursResult<D>
    where D: Deserialize
  {
    self.get(endpoint, self.add_key(params))
  }

  pub fn orderbook(&self) -> CursResult<OrderBook> {
    self.get("btc_usd/market/order_book", vec![])
  }

  //pub fn transactions() -> CursResult<

  pub fn profile(&self) -> CursResult<Profile> {
    self.private_get("private/profile", vec![])
  }

  pub fn orders(&self) -> CursResult<Vec<Order>> {
    self.private_get("private/orders", vec![])
  }

  pub fn bids(&self) -> BidsApi {
    BidsApi{api: self.clone()}
  }

  pub fn asks(&self) -> AsksApi {
    AsksApi{api: self.clone()}
  }
}

