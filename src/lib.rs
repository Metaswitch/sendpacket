#![macro_use]
#![allow(unused_macros)]

#[macro_use]
extern crate derive_new;
extern crate ipnetwork;
extern crate pnet;

use std::ops::Div;
use std::str::FromStr;
use std::num::ParseIntError;
use std::fs;
use std::io::Read;
use std::path::Path;

use pnet::datalink::{Channel, NetworkInterface, MacAddr};
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::{Packet, MutablePacket};

///
/// Macros
///
#[macro_export]
macro_rules! ip {
  ( $( $k:ident=$v:expr ),* ) => {{
    Ip {
      $(
        $k: $v.into(),
      )*
    }
  }};
}

#[macro_export]
macro_rules! mpls {
  ( $( $k:ident=$v:expr ),* ) => {{
    MPLS {
      $(
        $k: $v.into(),
      )*
    }
  }};
}

#[macro_export]
macro_rules! tcp {
  ( $( $k:ident=$v:expr ),* ) => {{
    Tcp{
      $(
        $k: $v.into(),
      )*
    }
  }};
}

#[macro_export]
macro_rules! mac {
  ( $( $K:ident=$v:expr ),* ) => {{
    Mac {
      $(
        $K: $v.into(),
      )*
      ..Default::default()
    }
  }};
}

#[macro_export]
macro_rules! ether {
    ( $( $k:ident=$v:expr ),* ) => {{
        Ether {
            $(
                $k: $v.into(),
            )*
        }
    }};
}

#[macro_export]
macro_rules! payload {
    ( $( $v:expr ),* ) => {{
        Payload::new(
            $(
                $v.into(),
            )*
        )
    }};

    ( $( $k:ident=$v:expr ),* ) => {{
        Payload {
            $(
                $k: $v.into(),
            )*
        }
    }};
}

///
/// Basic type system
///


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mac {
    address: [u8; 6]
}

impl From<[u8; 6]> for Mac{
    fn from(address: [u8; 6]) -> Self {
        Mac { address }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct Ether {
    pub src_mac: Mac,
    pub dst_mac: Mac,
}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct MPLS {
    pub label: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ip {
    pub src: String,
    pub dst: String
}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct Tcp {
    pub dport: u16,
    pub sport: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct Payload {
    pub payload: Vec<u8>,
}


///
/// Encapsulation types
///

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct L2 {
    ether: Ether,
    mpls_labels: Vec<MPLS>,
}

impl PackageHeader for Ether {
    fn build_header(&mut self, payload: &[u8]) -> Vec<u8> {
        // Insert RLC function here for L2 packets
        vec![]
    }
}

impl PackageHeader for L2 {
    fn build_header(&mut self, payload: &[u8]) -> Vec<u8> {
        let l2_packet: Vec<u8> = vec![];
        // Insert RLC function here for L2 packets

        self.ether.build_header(&l2_packet)
    }
}

impl Div<MPLS> for Ether {
    type Output = L2;

    fn div(self, rhs: MPLS) -> Self::Output {
        L2{
            ether: self,
            mpls_labels: vec![rhs],
        }
    }
}

impl Div<MPLS> for L2 {
    type Output = L2;

    fn div(mut self, rhs: MPLS) -> Self::Output {
        self.mpls_labels.push(rhs);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct L3 {
    l2: L2,
    ip: Ip,
}

impl PackageHeader for L3 {
    fn build_header(&mut self, payload: &[u8]) -> Vec<u8> {
        let l3_packet: Vec<u8> = vec![];
        // Insert RLC function here for L3 packets

        self.l2.build_header(&l3_packet)
    }
}

impl Div<Ip> for L2 {
    type Output = L3;

    fn div(self, rhs: Ip) -> Self::Output {
        L3{
            l2: self,
            ip: rhs,
        }
    }
}


impl Div<Ip> for Ether {
    type Output = L3;

    fn div(self, rhs: Ip) -> Self::Output {
        L3 {
            l2: L2{
                    ether: self,
                    mpls_labels: vec![],
                },
            ip: rhs,
        }
    }
}

pub trait Transport {}

impl Transport for Tcp {}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct L3Over<T: Transport> {
    l3: L3,
    transport: T,
}

impl PackageHeader for L3Over<Tcp> {
    fn build_header(&mut self, payload: &[u8]) -> Vec<u8> {
        let l3_over_tcp_packet: Vec<u8> = vec![];
        // Insert RLC function here for L3 over TCP packets

        self.l3.build_header(&l3_over_tcp_packet)
    }
}

impl<T: Transport> Div<T> for L3 {
    type Output = L3Over<T>;

    fn div(self, rhs: T) -> Self::Output {
        L3Over::new(self, rhs)
    }
}

pub trait PackageHeader {
    fn build_header(&mut self, payload: &[u8]) -> Vec<u8>;
}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct Package<H: PackageHeader> {
    header: H,
    payload: Payload,
}

impl<H: PackageHeader> Package<H> {
    fn build_packet(&mut self) -> Vec<u8> {
        self.header.build_header(&self.payload.payload)
    }
}

macro_rules! payload_div {
    ($header:ty) => {
        impl Div<Payload> for $header {
            type Output = Package<Self>;

            fn div(self, rhs: Payload) -> Self::Output {
                Package {
                    payload: rhs,
                    header: self,
                }
            }
        }
    }
}

payload_div!(Ether);
payload_div!(L2);
payload_div!(L3);
payload_div!(L3Over<Tcp>);

impl FromStr for Mac {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let octets: Vec<&str> = s.split(":").collect();
        assert_eq!(octets.len(), 6);
        let octets = octets.iter()
            .map(|p|u8::from_str_radix(p, 16))
            .collect::<Result<Vec<u8>, Self::Err>>();
        println!("Mac octets: {:?}", octets);

        match octets {
            Err(error) => Err(error),
            Ok(octs) => {
                let mut oct_array: [u8; 6] = [0; 6];
                oct_array.clone_from_slice(&octs);

                Ok(Mac {address: oct_array})
            }
        }
    }
}

impl Default for Mac {
    fn default() -> Mac {
        let net = Path::new("/sys/class/net");
        let entry = fs::read_dir(net).expect("Error");

        let ifaces = entry.filter_map(|p| p.ok())
                          .map(|p| p.path().file_name().expect("Error").to_os_string())
                          .filter_map(|s| s.into_string().ok())
                          .collect::<Vec<String>>();
        println!("Available interfaces: {:?}", ifaces);

        // TODO: currently just takes the first network interface it sees. What should it actually use???
        let iface = net.join(ifaces[0].as_str()).join("address");
        let mut f = fs::File::open(iface).expect("Failed");
        let mut macaddr = String::new();
        f.read_to_string(&mut macaddr).expect("Error");

        Mac::from_str(&macaddr.trim()).unwrap()
    }
}


///
/// Tests
///

#[cfg(test)]
mod tests {
  use Ip;
  use Tcp;
  use Mac;

  #[test]
  fn macro_ip_works() {
    assert_eq!(Ip {dst: "hello".into()}, ip!(dst="hello"));
  }

  #[test]
  fn macro_tcp_works() {
    assert_eq!(Tcp {}, tcp!());
  }

  #[test]
  fn macro_mac_works() {
    assert_eq!(Mac {address: [0; 6]}, mac!(address=[0; 6]));
  }

  #[test]
  fn macro_default_mac_works() {
    let mac: Mac = Default::default();
    assert_eq!(mac, mac!());
  }

  #[test]
  fn macro_tcp_ip_div_fv() {
    assert_eq!(Ip {dst: "hello".into()} / Tcp {},
               ip!(dst="hello")/tcp!());
  }
}

mod builder {
    use L2;
    use Ip;
    use Tcp;

    fn build_ethernet_pkt(ether: L2, payload: [u8; 1024]) -> [u8; 1024] {
        // TODO implement!
        panic!("Not implemented");
        payload
    }

    fn build_ip_pkt(ip: Ip, ethernet_pkt: [u8; 1024]) -> [u8; 1024] {
        // TODO implement!
        panic!("Not implemented");
        ethernet_pkt
    }

    fn build_tcp_pkt(tcp: Tcp, ip_pkt: [u8; 1024]) -> [u8; 1024] {
        // TODO implement!
        panic!("Not implemented");
        ip_pkt
    }
}

