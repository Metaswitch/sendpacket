#![macro_use]
#![allow(unused_macros)]

#[macro_use]
extern crate derive_new;
extern crate ipnetwork;
extern crate pnet;

use ipnetwork::IpNetwork;
use pnet::packet::{MutablePacket, Packet};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperation, ArpOperations};
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ip;
use pnet::packet::ipv4;
use std::fs;
use std::io::Read;
use std::net::Ip;
use std::num::ParseIntError;
use std::ops::Div;
use std::path::Path;
use std::str::FromStr;


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

trait EtherTypeable {
    fn ether_type(&self) -> EtherType;
}

impl EtherTypeable for MPLS {
    fn ether_type(&self) -> EtherType {
        EtherTypes::Mpls
    }
}

impl EtherTypeable for Ip {
    fn ether_type(&self) -> EtherType {
        match self.src.parse::<IpNetwork>().unwrap() {
            IpNetwork::V4(_) => EtherTypes::Ipv4,
            IpNetwork::V6(_) => EtherTypes::Ipv6,
        }
    }
}


///
/// Encapsulation types
///

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct L2 {
    ether: Ether,
    mpls_labels: Vec<MPLS>,
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
    ip: Ip, // if you allow more types here, they must implement EtherTypeable
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

impl<T: Transport> Div<T> for L3 {
    type Output = L3Over<T>;

    fn div(self, rhs: T) -> Self::Output {
        L3Over::new(self, rhs)
    }
}

impl L3Over<Tcp> {
    fn mutable_packet(&self, payload: Vec<u8>) -> Box<MutablePacket> {
        let mut ethernet_buffer = [0u8; 42];
        let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

        ethernet_packet.set_destination(self.l3.l2.ether.src_mac);
        ethernet_packet.set_source(self.l3.l2.ether.dst_mac);
        ethernet_packet.set_ethertype(self.l3.ip.ether_type());

        let ipv4_struct = ipv4::Ipv4 {
            version: 4,
            header_length: 5,
            dscp: 0,
            ecn: 0,
            total_length: 127, // todo
            identification: 0,
            flags: 0,
            fragment_offset: 0,
            ttl: 1,
            next_level_protocol: ip::IpNextHeaderProtocols::Tcp,
            checksum: 123, // todo
            source: self.l3.ip.src.parse().unwrap(),
            destination: self.l3.ip.dst.parse().unwrap(),
            options: vec![],
            payload: self.transport.make_payload(payload)
        };


        let mut ip_packet = ipv4::Ipv4Packet::new();


        let mut arp_buffer = [0u8; 28];
        let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();

        arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
        arp_packet.set_protocol_type(EtherTypes::Ipv4);
        arp_packet.set_hw_addr_len(6);
        arp_packet.set_proto_addr_len(4);
        arp_packet.set_operation(arp_operation);
        arp_packet.set_sender_hw_addr(source_mac);
        arp_packet.set_sender_proto_addr(source_ip);
        arp_packet.set_target_hw_addr(target_mac);
        arp_packet.set_target_proto_addr(target_ip);

        ethernet_packet.set_payload(arp_packet.packet_mut())
    }
}

// pub trait InsideL3 {}
pub trait PackageHeader {}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct Package<H: PackageHeader> {
    header: H,
    payload: Payload,
}

macro_rules! payload_div {
    ($header:ty) => {
        impl PackageHeader for $header {}

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
        let net = Path::new("/sys/class/net");
        let entry = fs::read_dir(net).expect("Error");

        let ifaces = entry.filter_map(|p| p.ok())
                          .map(|p| p.path().file_name().expect("Error").to_os_string())
                          .filter_map(|s| s.into_string().ok())
                          .collect::<Vec<String>>();
        println!("Available interfaces: {:?}", ifaces);

        // TODO: currently just takes the first network interface it sees... What should it actually use???
        let iface = net.join(ifaces[0].as_str()).join("address");
        let mut f = fs::File::open(iface).expect("Failed");
        let mut macaddr = String::new();
        f.read_to_string(&mut macaddr).expect("Error");

        macaddr.trim().parse().unwrap()
    }
}


///
/// Tests
///

mod tests {
    use Ether;
    use Ip;
    use Mac;
    use Tcp;

    #[test]
  fn macro_ip_works() {
    assert_eq!(Ip {src: "".into(), dst: "hello".into()}, ip!(src="", dst="hello"));
  }

  #[test]
  fn macro_tcp_works() {
    assert_eq!(Tcp {dport: 0, sport: 1}, tcp!(dport=0u16, sport=1u16));
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
    assert_eq!(Ether {src_mac: [10,1,1,1,1,1].into(), dst_mac: [10,1,1,1,1,2].into()} / Ip {src: "".into(), dst: "hello".into()} / Tcp {dport: 0u16, sport: 1u16},
               ether!(src_mac = [10,1,1,1,1,1], dst_mac = [10,1,1,1,1,2]) / ip!(src="", dst="hello")/tcp!(dport=0u16, sport=1u16));
  }
}
