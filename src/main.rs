#[macro_use]
extern crate lib_hack12;
use lib_hack12::*;

fn main() {
    let p = l2!(src_mac = [10,1,1,1,1,1], dst_mac = [10,1,1,1,1,2]) / mpls!(label = 77) / ip!(dst = "127.0.0.1", src = "10.8.0.1") / tcp!(dport = 85, sport = 10);
    println!("{:?}", p);
}
