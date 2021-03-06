#[macro_use]
extern crate bitex;
extern crate http_stub;

use std::{thread, time};
use http_stub as hs;
use bitex::{Api, OrderBook, Transaction, Profile, Order, Bid, Ask};

#[test]
fn gets_orderbook(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Get);
    stub.got_path("/api-v1/rest/btc_usd/market/order_book");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"{"bids":[[500.0,1],[490.0,2]], "asks":[[510.0,1],[520.0,2]]}"#);
  });
  thread::sleep(time::Duration::from_millis(100));

  let OrderBook{bids, asks} = Api::new(&url).orderbook().unwrap();
  assert_eq!(bids[0], (500.0, 1.0));
  assert_eq!(asks[1], (520.0, 2.0));
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
  thread::sleep(time::Duration::from_millis(100));

  let ts : Vec<Transaction> = Api::new(&url).transactions().unwrap();

  assert_eq!(ts[0].amount, 0.01119999);
  assert_eq!(ts[1].amount, 0.01100000);
}

#[test]
fn gets_profile(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Get);
    stub.got_path(r"/api-v1/rest/private/profile\?api_key=bogus");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"{
      "usd_balance": 10000.00,
      "usd_reserved": 2000.00,
      "usd_available": 8000.00,
      "btc_balance": 20.00000000,
      "btc_reserved": 5.00000000,
      "btc_available": 15.00000000,
      "fee": 0.5,
      "btc_deposit_address": "1ABCD",
      "more_mt_deposit_code": "BITEX0000000"
    }"#);
  });
  thread::sleep(time::Duration::from_millis(100));

  let profile: Profile = Api::new(&url).key("bogus").profile().unwrap();

  assert_eq!(profile, Profile{
    usd_balance: 10000.00,
    usd_reserved: 2000.00,
    usd_available: 8000.00,
    btc_balance: 20.00,
    btc_reserved: 5.00,
    btc_available: 15.00,
    fee: 0.5,
    btc_deposit_address: "1ABCD".to_string(),
    more_mt_deposit_code: "BITEX0000000".to_string(),
  });
}

#[test]
fn gets_orders(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Get);
    stub.got_path(r"/api-v1/rest/private/orders\?api_key=bogus");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"[
      [1, 12345678, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01],
      [2, 12345678, 946685400, 1, 200.00, 20.00, 2000.00, 1, 0, 1.1, "ApiKey#2", 0.01]
    ]"#);
  });
  thread::sleep(time::Duration::from_millis(100));

  let orders: Vec<Order> = Api::new(&url).key("bogus").orders().unwrap();

  assert_eq!(orders, [
    Order::Bid(Bid{
      id: 12345678,
      creation: 946685400,
      orderbook: 1,
      amount_to_spend: 100.0,
      remaining_amount: 10.0,
      price: 1000.0,
      status: 1,
      cancelation_reason: 0,
      produced_amount: 1.1,
      issuer: Some("ApiKey#1".to_string()),
      fees_paid: 0.01
    }),
    Order::Ask(Ask{
      id: 12345678,
      creation: 946685400,
      orderbook: 1,
      amount_to_spend: 200.0,
      remaining_amount: 20.0,
      price: 2000.0,
      status: 1,
      cancelation_reason: 0,
      produced_amount: 1.1,
      issuer: Some("ApiKey#2".to_string()),
      fees_paid: 0.01
    })
  ]);
}

#[test]
fn places_a_bid(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Post);
    stub.got_path(r"/api-v1/rest/private/bids");
    stub.got_body(r"amount=100");
    stub.got_body(r"price=10");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"
      [1, 12345678, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01]
    "#);
  });
  thread::sleep(time::Duration::from_millis(100));

  let bid: Bid = Api::new(&url).key("bogus").bids().create(100.0, 10.0).unwrap();

  assert_eq!(bid, Bid{
    id: 12345678,
    creation: 946685400,
    orderbook: 1,
    amount_to_spend: 100.0,
    remaining_amount: 10.0,
    price: 1000.0,
    status: 1,
    cancelation_reason: 0,
    produced_amount: 1.1,
    issuer: Some("ApiKey#1".to_string()),
    fees_paid: 0.01
  });
}

#[test]
fn finds_a_bid(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Get);
    stub.got_path(r"/api-v1/rest/private/bids/1");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"
      [1, 12345678, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01]
    "#);
  });
  thread::sleep(time::Duration::from_millis(100));

  let bid: Bid = Api::new(&url).key("bogus").bids().show(1).unwrap();

  assert_eq!(bid, Bid{
    id: 12345678,
    creation: 946685400,
    orderbook: 1,
    amount_to_spend: 100.0,
    remaining_amount: 10.0,
    price: 1000.0,
    status: 1,
    cancelation_reason: 0,
    produced_amount: 1.1,
    issuer: Some("ApiKey#1".to_string()),
    fees_paid: 0.01
  });
}

#[test]
fn cancels_a_bid(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Post);
    stub.got_path(r"/api-v1/rest/private/bids/1/cancel");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"
      [1, 12345678, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01]
    "#);
  });
  thread::sleep(time::Duration::from_millis(100));

  let bid: Bid = Api::new(&url).key("bogus").bids().cancel(1).unwrap();

  assert_eq!(bid, Bid{
    id: 12345678,
    creation: 946685400,
    orderbook: 1,
    amount_to_spend: 100.0,
    remaining_amount: 10.0,
    price: 1000.0,
    status: 1,
    cancelation_reason: 0,
    produced_amount: 1.1,
    issuer: Some("ApiKey#1".to_string()),
    fees_paid: 0.01
  });
}

#[test]
fn places_a_ask(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Post);
    stub.got_path(r"/api-v1/rest/private/asks");
    stub.got_body(r"amount=100");
    stub.got_body(r"price=10");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"
      [2, 12345678, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01]
    "#);
  });
  thread::sleep(time::Duration::from_millis(100));

  let ask: Ask = Api::new(&url).key("bogus").asks().create(100.0, 10.0).unwrap();

  assert_eq!(ask, Ask{
    id: 12345678,
    creation: 946685400,
    orderbook: 1,
    amount_to_spend: 100.0,
    remaining_amount: 10.0,
    price: 1000.0,
    status: 1,
    cancelation_reason: 0,
    produced_amount: 1.1,
    issuer: Some("ApiKey#1".to_string()),
    fees_paid: 0.01
  });
}

#[test]
fn finds_an_ask(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Get);
    stub.got_path(r"/api-v1/rest/private/asks/1");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"
      [2, 12345678, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01]
    "#);
  });
  thread::sleep(time::Duration::from_millis(100));

  let ask: Ask = Api::new(&url).key("bogus").asks().show(1).unwrap();

  assert_eq!(ask, Ask{
    id: 12345678,
    creation: 946685400,
    orderbook: 1,
    amount_to_spend: 100.0,
    remaining_amount: 10.0,
    price: 1000.0,
    status: 1,
    cancelation_reason: 0,
    produced_amount: 1.1,
    issuer: Some("ApiKey#1".to_string()),
    fees_paid: 0.01
  });
}

#[test]
fn cancels_an_ask(){
  let url = hs::HttpStub::run(|mut stub|{
    stub.got_method(hs::Method::Post);
    stub.got_path(r"/api-v1/rest/private/asks/1/cancel");
    stub.send_header(hs::header::ContentType(
      hs::Mime(hs::TopLevel::Application, hs::SubLevel::Json, vec![])));
    stub.send_body(r#"
      [2, 12345678, 946685400, 1, 100.00, 10.00, 1000.00, 1, 0, 1.1, "ApiKey#1", 0.01]
    "#);
  });
  thread::sleep(time::Duration::from_millis(100));

  let ask: Ask = Api::new(&url).key("bogus").asks().cancel(1).unwrap();

  assert_eq!(ask, Ask{
    id: 12345678,
    creation: 946685400,
    orderbook: 1,
    amount_to_spend: 100.0,
    remaining_amount: 10.0,
    price: 1000.0,
    status: 1,
    cancelation_reason: 0,
    produced_amount: 1.1,
    issuer: Some("ApiKey#1".to_string()),
    fees_paid: 0.01
  });
}

