#[macro_use]
extern crate lib_hack12;
extern crate pnet;
extern crate pnet_datalink;

use lib_hack12::*;

use std::net::{Ipv4Addr, AddrParseError};

use pnet::datalink::{Channel, NetworkInterface, MacAddr};
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::{Packet, MutablePacket};

fn main() {
//    let p = ether!(src_mac = [10,1,1,1,1,1], dst_mac = [10,1,1,1,1,2]) / mpls!(label = 77u32) / ip!(dst = "127.0.0.1", src = "10.8.0.1") / tcp!(dport = 85u16, sport = 10u16) / payload!(vec![1,2,3,4]);
//    println!("{:#?}", p);

    let interface = pnet::datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == "enp0s8")
        .unwrap();

    let source_mac = interface.mac_address();
    let dest_mac = MacAddr::new(0x08, 0x00, 0x27, 0x63, 0x9d, 0xc2);

    let packet = ether!(src_mac = [10, 1, 1, 1, 1, 1], dst_mac = [10, 1, 1, 1, 1, 2]) / payload!("hello".to_string().into_bytes());

    println!("Made packet {:#?}", packet);

    packet.send(&interface);
    println!("Sent");

    let rcv_pkt = packet.recv(&interface);
    println!("Received: {:?}", rcv_pkt);
}

//fn send_l2_packet(interface: NetworkInterface,
//                  source_mac: MacAddr,
//                  target_mac: MacAddr,
//                  payload: &[u8]) {
//    let(mut tx, _) = match pnet::datalink::channel(&interface, Default::default()) {
//        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
//        Ok(_) => panic!("Unknown channel type"),
//        Err(e) => panic!("Error happened {}", e),
//    };
//
//    let mut ethernet_buffer = [0u8; 42];
//    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();
//
//    ethernet_packet.set_destination(target_mac);
//    ethernet_packet.set_source(source_mac);
//    ethernet_packet.set_ethertype(EtherTypes::Arp);
//
//    ethernet_packet.set_payload(payload);
//
//    tx.send_to(ethernet_packet.packet(), Some(interface));
//}
//
//fn send_ip_packet(interface: NetworkInterface,
//                  source_mac: MacAddr,
//                  target_mac: MacAddr,
//                  source_ip: Ipv4Addr,
//                  target_ip: Ipv4Addr,
//                  payload: &[u8]) {
//
//
//    let mut ethernet_buffer = [0u8; 200];
//    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();
//
//    ethernet_packet.set_destination(target_mac);
//    ethernet_packet.set_source(source_mac);
//    ethernet_packet.set_ethertype(EtherTypes::Ipv4);
//
//    let mut ipv4_buffer = [0u8; 100];
//    let mut ipv4_packet = MutableIpv4Packet::new(&mut ipv4_buffer).unwrap();
//
//    ipv4_packet.set_source(source_ip);
//    ipv4_packet.set_destination(target_ip);
//    ipv4_packet.set_ttl(45);
//    ipv4_packet.set_version(4);
//    ipv4_packet.set_total_length(100);
//    ipv4_packet.set_header_length(24);
//
//    ipv4_packet.set_payload(payload);
//
//    ethernet_packet.set_payload(ipv4_packet.packet_mut());
//
//    tx.send_to(ethernet_packet.packet(), Some(interface));
//}
