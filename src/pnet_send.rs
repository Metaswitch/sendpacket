extern crate pnet;

use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel;
use pnet::packet::ethernet::{MutableEthernetPacket, Ethernet, EtherType, EtherTypes};

use std::env;

fn main() {
    let src_if_name = env::args().nth(1)
        .expect("Usage: ./test_pnet <src interface> <dst interface>>");
    let dst_if_name = env::args().nth(2)
        .expect("Usage: ./test_pnet <src interface> <dst interface>>");

    let src_if = find_interface(src_if_name);
    let dst_if = find_interface(dst_if_name);

    // Create a new channel, dealing with layer 2 packets
    let (mut tx, mut rx) = match datalink::channel(&src_if, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    println!("sending from {} to {}", src_if.mac.unwrap(), dst_if.mac.unwrap());
    tx.build_and_send(
        1,
        MutableEthernetPacket::packet_size(&pkt_contents),
        &mut |mut new_packet| {
            let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();
            new_packet.set_destination(dst_if.mac.unwrap());
            new_packet.set_source(src_if.mac.unwrap());
            new_packet.set_ethertype(EtherTypes::Ipv4);
            new_packet.set_payload(&vec![12, 31, 1, 3]);
        }
    );

    println!("Sent");

    match rx.next() {
        Ok(packet) => {
            println!("Received: {:?}", packet);
        },
        Err(_) => {
            panic!("At the disco");
        }
    }
}

fn find_interface(interface_name: String) -> NetworkInterface {
    let interface_names_match =
        |iface: &NetworkInterface| iface.name == interface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    interfaces.into_iter()
              .filter(interface_names_match)
              .next()
              .expect("Could not open interface")
}

