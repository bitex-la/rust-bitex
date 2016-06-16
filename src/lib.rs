#![feature(custom_derive, plugin, macro_reexport)]
#![plugin(serde_macros)]

#[macro_use]
pub extern crate curs;
extern crate mime_guess;

pub use curs::hyper::client::response::Response;
pub use curs::hyper::method::Method;
use curs::{CursResult, DecodableResult, Params};
use curs::serde::{Deserialize, Deserializer};
use curs::serde::d128;
use curs::serde::de::impls::{TupleVisitor4, TupleVisitor12};
pub use curs::hyper::status::StatusCode;
pub use curs::hyper::header::{Headers, Header, HeaderFormat, UserAgent, ContentType};

#[derive(Debug, Clone)]
pub struct Api<'a> {
  key: &'a str,
  url_base: &'a str
}

type OrderVisitor =
  TupleVisitor12<i64, i64, i64, i64, d128, d128, d128, i64, i64, d128, Option<String>, d128>;

type OrderTuple =
  (i64, i64, i64, i64, d128, d128, d128, i64, i64, d128, Option<String>, d128);

macro_rules! make_order_endpoint {
  ($name:ident, $api:ident, $order_type_value:expr, $endpoint_name:expr) => (
    #[derive(Debug)]
    pub struct $api<'a> {
      api: Api<'a>
    }

    impl <'a> $api<'a> {
      pub fn show(&self, id: i64) -> CursResult<$name> {
        self.api.private_get(&*format!("private/{}/{}", $endpoint_name, id), vec![])
      }

      pub fn create(&self, amount: d128, price: d128) -> CursResult<$name> {
        self.api.private_post(&*format!("private/{}", $endpoint_name),
          vec![("amount", &*amount.to_string()), ("price", &*price.to_string())])
      }

      pub fn cancel(&self, id: i64) -> CursResult<$name> {
        self.api.private_post(&*format!("private/{}/{}/cancel", $endpoint_name, id), vec![])
      }
    }

    #[derive(Debug, PartialEq)]
    pub struct $name {
      pub id: i64,
      pub creation: i64,
      pub orderbook: i64,
      pub amount_to_spend: d128,
      pub remaining_amount: d128,
      pub price: d128,
      pub status: i64,
      pub cancelation_reason: i64,
      pub produced_amount: d128,
      pub issuer: Option<String>,
      pub fees_paid: d128
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

#[derive(Debug, PartialEq)]
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
  pub bids: Vec<(d128, d128)>,
  pub asks: Vec<(d128, d128)>
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Transaction {
  pub timestamp: i64,
  pub id: i64,
  /// The price that was paid per bitcoin.
  pub price: d128,
  /// The bitcoin amount sold.
  pub amount: d128
}

type TransactionVisitor = TupleVisitor4<i64, i64, d128, d128>; 

impl Deserialize for Transaction {
  fn deserialize<D: Deserializer>(d: &mut D) -> Result<Transaction, D::Error>{
    let tuple = try!(d.deserialize(TransactionVisitor::new()));
    Ok(Transaction{ timestamp: tuple.0, id: tuple.1, price: tuple.2, amount: tuple.3 })
  }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Profile {
  pub usd_balance: d128,
  pub usd_reserved: d128,
  pub usd_available: d128,
  pub btc_balance: d128,
  pub btc_reserved: d128,
  pub btc_available: d128,
  pub fee: d128,
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
    Api{ key: "", url_base: SANDBOX_URL_BASE}
  }

  // Set the current API key for authenticated requests.
  pub fn key(mut self, key: &'a str) -> Api {
    self.key = key;
    self
  }

  fn url(&self, endpoint: &str) -> String {
    format!("{}/api-v1/rest/{}", self.url_base, endpoint)
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

  pub fn transactions(&self) -> CursResult<Vec<Transaction>> {
    self.get("btc_usd/market/transactions", vec![])
  }

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

