#[macro_use]
extern crate decimal;
extern crate bitex;
extern crate http_stub;

use http_stub as hs;
use bitex::{Api, OrderBook, Transaction};

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
  assert_eq!(bids[0], (d128!(500), d128!(1)));
  assert_eq!(asks[1], (d128!(520), d128!(2)));
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

  assert_eq!(ts[0].amount, d128!(0.01119999));
  assert_eq!(ts[1].amount, d128!(0.01100000));
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
