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

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct Tcp;

#[derive(Clone, Debug, PartialEq, Eq, new)]
pub struct IpOverTcp {
    dst: String,
}

impl Div<Tcp> for Ip {
    type Output = IpOverTcp;

    fn div(self, _rhs: Tcp) -> Self::Output {
        IpOverTcp {
            dst: self.dst
        }
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