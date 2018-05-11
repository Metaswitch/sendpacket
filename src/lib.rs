#![macro_use]
#![allow(unused_macros)]

#[macro_use]
extern crate derive_new;
extern crate ipnetwork;
extern crate pnet;

use ipnetwork::IpNetwork;
use pnet::datalink::{Channel, NetworkInterface, MacAddr, DataLinkReceiver, DataLinkSender};
use pnet::packet::{MutablePacket, Packet};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperation, ArpOperations};
use pnet::packet::ethernet::{EtherType, EtherTypes};
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ip::{self, IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::{self, MutableIpv4Packet};
use pnet::packet::udp::{ipv4_checksum, MutableUdpPacket};

use std::fs;
use std::io::Read;
use std::num::ParseIntError;
use std::ops::Div;
use pnet::datalink;
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
macro_rules! udp {
  ( $( $k:ident=$v:expr ),* ) => {{
    Udp{
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
pub struct EtherWrapper(Ether);

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
        EtherTypes::Ipv4
    }
}

pub struct DataLinkSession {
    interface: NetworkInterface,
}

impl DataLinkSession {
    pub fn new(if_name: &String) -> DataLinkSession {
        let interface = DataLinkSession::get_interface(if_name);

        DataLinkSession {
            interface: interface,
        }
    }

    fn get_interface(if_name: &String) -> NetworkInterface {
        pnet::datalink::interfaces()
            .into_iter()
            .find(|iface| iface.name == *if_name)
            .expect("Could not open specified interface")
    }

    pub fn local_mac_address(&self) -> Mac {
        Mac::from_mac_addr(self.interface.mac.unwrap())
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

impl Ether {
    fn build_header_inner(&self, payload: &[u8], ether_type: Option<EtherType>) -> Vec<u8> {
        let ether_buffer_len = payload.len() + 38; // 42 if with 802.1Q tags
        let mut ether_buffer = vec![0u8; ether_buffer_len];
        let mut ether_packet = MutableEthernetPacket::new(&mut ether_buffer).unwrap();

        ether_packet.set_source(self.src_mac.to_mac_addr());
        ether_packet.set_destination(self.dst_mac.to_mac_addr());

        if let Some(t) = ether_type {
            ether_packet.set_ethertype(t)
        }

        ether_packet.set_payload(payload);
        ether_packet.packet().to_vec()
    }
}

impl PackageHeader<EtherType> for Ether {
    fn build_header(&self, payload: &[u8], ether_type: EtherType) -> Vec<u8> {
        self.build_header_inner(payload, Some(ether_type))
    }
}

impl PackageHeader<()> for Ether {
    fn build_header(&self, payload: &[u8], _extra: ()) -> Vec<u8> {
        self.build_header_inner(payload, None)
    }
}

impl PackageHeader<EtherType> for L2 {
    fn build_header(&self, payload: &[u8], ether_type: EtherType) -> Vec<u8> {
        let l2_packet: Vec<u8> = vec![];
        // Insert RLC function here for L2 packets

        self.ether.build_header(&l2_packet, ether_type)
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
    ip: Ip, // if you allow more types here, they must implement EtherTypeable
}

impl PackageHeader<IpNextHeaderProtocol> for L3 {
    fn build_header(&self, payload: &[u8], next_header: IpNextHeaderProtocol) -> Vec<u8> {
        let l3_len = payload.len() + 20;
        let l3_buffer = vec![0u8; l3_len];
        let mut l3_packet = MutableIpv4Packet::owned(l3_buffer).unwrap();

        l3_packet.set_version(4);
        l3_packet.set_header_length(5);
        l3_packet.set_total_length(l3_len as u16);
        l3_packet.set_ttl(1);
        l3_packet.set_next_level_protocol(next_header);

        l3_packet.set_source(self.ip.src.parse().unwrap());
        l3_packet.set_destination(self.ip.dst.parse().unwrap());
        l3_packet.set_payload(payload);

        let checksum = {
            let imm_packet = l3_packet.to_immutable();
            ipv4::checksum(&imm_packet)
        };

        l3_packet.set_checksum(checksum);
        self.l2.build_header(l3_packet.packet(), self.ip.ether_type())
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
pub struct Udp {
    pub src_port: u16,
    pub dst_port: u16,
}

impl Transport for Udp {}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct L3Over<T: Transport> {
    l3: L3,
    transport: T,
}

impl PackageHeader<()> for L3Over<Tcp> {
    fn build_header(&self, payload: &[u8], _extra: ()) -> Vec<u8> {
        let l3_over_tcp_packet: Vec<u8> = vec![];
        // Insert RLC function here for L3 over TCP packets
        panic!("At the disco");
        self.l3.build_header(&l3_over_tcp_packet, IpNextHeaderProtocols::Tcp)
    }
}


impl PackageHeader<()> for L3Over<Udp> {
    fn build_header(&self, payload: &[u8], _extra: ()) -> Vec<u8> {
        let l3_over_udp_packet: Vec<u8> = vec![];


        let udp_len = payload.len() + 8;
        let mut udp_buffer = vec![0u8; udp_len];
        let mut udp_packet = MutableUdpPacket::new(&mut udp_buffer).unwrap();

        udp_packet.set_source(self.transport.src_port);
        udp_packet.set_destination(self.transport.dst_port);
        udp_packet.set_length(udp_len as u16);
        udp_packet.set_payload(payload);

        // Checksum is optional for UDP
        //    let checksum = {
        //       ipv4_checksum(upd_packet.to_immutable(), self.l3.ip.src.parse().unwrap(), self.l3.ip.dst.parse().unwrap())
        //       let imm_packet = l3_packet.to_immutable();
        //       ipv4::checksum(&imm_packet)
        //    };

        udp_packet.set_checksum(0);

        self.l3.build_header(&udp_packet.packet(), IpNextHeaderProtocols::Udp)
    }
}

impl<T: Transport> Div<T> for L3 {
    type Output = L3Over<T>;

    fn div(self, rhs: T) -> Self::Output {
        L3Over::new(self, rhs)
    }
}

pub trait PackageHeader<ExtraInfo> {
    fn build_header(&self, payload: &[u8], ether_type: ExtraInfo) -> Vec<u8>;
}

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct Package<H: PackageHeader<()>> {
    header: H,
    payload: Payload,
}

impl<H: PackageHeader<()>> Package<H> {
    fn build_packet(&self) -> Vec<u8> {
        self.header.build_header(&self.payload.payload, ())
    }

    pub fn send(&self, session: &DataLinkSession) {
        let (mut tx, mut rx) = match pnet::datalink::channel(&session.interface, Default::default()) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("Unknown channel type"),
            Err(e) => panic!("Error happened {}", e),
        };
        tx.send_to(&self.build_packet(), None);
    }

    pub fn recv(&self, session: &DataLinkSession) -> Vec<u8> {
        let (mut tx, mut rx) = match pnet::datalink::channel(&session.interface, Default::default()) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("Unknown channel type"),
            Err(e) => panic!("Error happened {}", e),
        };

        match rx.next() {
            Ok(packet) => {
                packet.to_vec()
            },
            Err(e) => {
                panic!("Error receiving packet: {}", e);
            }
        }
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
//payload_div!(L2);
//payload_div!(L3);
payload_div!(L3Over<Udp>);

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

impl Mac {
    fn to_mac_addr(&self) -> MacAddr {
        MacAddr::new(self.address[0],
                     self.address[1],
                     self.address[2],
                     self.address[3],
                     self.address[4],
                     self.address[5])
    }

    fn from_mac_addr(mac_addr: MacAddr) -> Mac {
        Mac {
            address: [mac_addr.0,
                       mac_addr.1,
                       mac_addr.2,
                       mac_addr.3,
                       mac_addr.4,
                       mac_addr.5]
        }
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
