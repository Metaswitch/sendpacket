#[macro_use]
extern crate lib_sendpacket;
use lib_sendpacket::*;

fn main() {
    let p = ether!(src_mac = [10,1,1,1,1,1], dst_mac = [10,1,1,1,1,2]) / mpls!(label = 77u32) / ip!(dst = "127.0.0.1", src = "10.8.0.1") / tcp!(dport = 85u16, sport = 10u16) / payload!(vec![1,2,3,4]);
    println!("{:#?}", p);
}
