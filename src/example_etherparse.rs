extern crate etherparse;
use etherparse::PacketBuilder;

fn main() {
    let builder = PacketBuilder::ethernet2([1,2,3,4,5,6],     //source mac
               [7,8,9,10,11,12]) //destionation mac
    .ipv4([192,168,1,1], //source ip
          [192,168,1,2], //desitionation ip
          20)            //time to live
    .udp(21,    //source port
         1234); //desitnation port

    //payload of the udp packet
    let payload = [1,2,3,4,5,6,7,8];

    let mut result = vec![];

    //serialize
    //this will automatically set all length fields, checksums and identifiers (ethertype & protocol)
    //before writing the packet out to "result"
    builder.write(&mut result, &payload).unwrap();
    println!("{:?}", result.into_iter().map(|x| format!("{:x}", x)).collect::<Vec<_>>());
}
