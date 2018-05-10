use std::ops::Div;

use std::str::FromStr;
use std::num::ParseIntError;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ip {
    dst: String
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tcp;

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(IpOverTcp { dst: "blah".to_string() }, Ip { dst: "blah".to_string()} / Tcp )
    }
}



#[derive(Debug, PartialEq)]
pub struct Mac {
    address: [u8; 6]
}

impl FromStr for Mac {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let octets: Vec<&str> = s.split(":").collect();
        assert_eq!(octets.len(), 6);
        let octets = octets.iter()
            .map(|p|u8::from_str_radix(p, 16).expect("Failed to parse octet"))
            .collect::<Vec<u8>>();
        println!("Octets: {:?}", octets);

        let mut oct_array: [u8; 6] = [0; 6];
        oct_array.clone_from_slice(&octets);

        Ok(Mac {address: oct_array})
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