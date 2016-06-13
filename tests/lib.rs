extern crate bitex;
extern crate http_stub;
#[macro_use]
//extern crate decimal;

use http_stub as hs;
use bitex::{Api, OrderBook, Transaction};
use std::str::FromStr;
//use decimal::d128;
use std::{f32, f64};
use bitex::curs::serde::d128;

#[test]
fn gets_orderbook(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Get);
    stub.got_path("/api-v1/rest/btc_usd/market/order_book");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"{"bids":[[500.0,1],[490.0,2]], "asks":[[510.0,1],[520.0,2]]}"#);
  });

  let OrderBook{bids, asks} = Api::new(&url).orderbook().unwrap();
  assert_eq!(bids[0], (d128::from_str("500.0").unwrap(), d128::from_str("1.0").unwrap()));
  //assert_eq!(asks[1], (d128::from_str("520.0").unwrap(), 2.0));
}

#[test]
fn gets_transactions(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Get);
    stub.got_path("/api-v1/rest/btc_usd/market/transactions");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"[
      [1461469200, 60644, 453.71391, 0.01119999],
      [1461469100, 60643, 453.71, 0.011]
    ]"#);
  });

  let ts : Vec<Transaction> = Api::new(&url).transactions().unwrap();

  assert_eq!((ts[0].amount * 100000000.0).round(), 1119999.0);
  assert_eq!((ts[1].amount * 100000000.0).round(), 1100000.0);
}

#[test]
fn gets_profile(){
}

#[test]
fn gets_orders(){
}

#[test]
fn places_a_bid(){
}

#[test]
fn finds_a_bid(){
}

#[test]
fn cancels_a_bid(){
}

#[test]
fn places_an_ask(){
}

#[test]
fn finds_an_ask(){
}

#[test]
fn cancels_an_ask(){
}
