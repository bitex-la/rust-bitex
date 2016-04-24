extern crate bitex;
extern crate http_stub;

use http_stub as hs;
use bitex::{Api, OrderBook};

#[test]
fn gets_orderbook(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Get);
    stub.got_path("/api-v1/rest/btc_usd/market/order_book");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"{"bids":[[500,1],[490,2]], "asks":[[510,1],[520,2]]}"#);
  });

  let OrderBook{bids, asks} = Api::new(&url).orderbook().unwrap();
  assert_eq!(bids[0], (500.0, 1.0));
  assert_eq!(asks[1], (520.0, 2.0));
}

#[test]
fn gets_transactions(){

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
