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
use std::marker::PhantomData;
use pnet::datalink;

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
  () => {{ Tcp {} }};
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


///
/// Type system
///

pub trait InsideL3 {}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct L2 {
    src_mac: Mac,
    dest_mac: Mac,
    vlan: Vec<()>,
}

impl InsideL3 for L2 {}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct MPLS<Inner: InsideL3> {
    label: String,
    inner: Inner,
}

impl<Inner: InsideL3> InsideL3 for MPLS<Inner> {}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct L3<Inner: InsideL3> {
    ip: ipnetwork::IpNetwork,
    inner: Inner,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ip {
    dst: String
}

pub trait Transport {}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct Tcp;

impl Transport for Tcp {}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct IpOver<T: Transport> {
    dst: String,
    #[new(default)] _phantom: PhantomData<T>,
}

impl<T: Transport> Div<T> for Ip {
    type Output = IpOver<T>;

    fn div(self, _rhs: T) -> Self::Output {
        IpOver::new(self.dst)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mac {
    address: [u8; 6]
}

impl FromStr for Mac {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let octets: Vec<&str> = s.split(":").collect();
        assert_eq!(octets.len(), 6);
        let octets = octets.iter()
            .map(|p| u8::from_str_radix(p, 16))
            .collect::<Result<Vec<u8>, Self::Err>>();
        println!("Mac octets: {:?}", octets);

        match octets {
            Err(error) => Err(error),
            Ok(octs) => {
                let mut oct_array: [u8; 6] = [0; 6];
                oct_array.clone_from_slice(&octs);

                Ok(Mac { address: oct_array })
            }
        }
    }
}

impl Default for Mac {
    fn default() -> Mac {
        // TODO: currently just takes the first network interface it sees. What should it actually use???
        let interface = datalink::interfaces()
            .into_iter()
            .next()
            .expect("Could not find any interfaces");
        let mac_addr = interface.mac.unwrap();
        Mac {address: [mac_addr.0, mac_addr.1, mac_addr.2, mac_addr.3, mac_addr.4, mac_addr.5]}
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
