use std::ops::Div;

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