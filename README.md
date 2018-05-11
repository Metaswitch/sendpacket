# sendpacket

[![Crates.io - Sendpacket](https://img.shields.io/crates/v/sendpacket.svg)](https://crates.io/crates/sendpacket) [![Build Status](https://travis-ci.org/Metaswitch/sendpacket.svg?branch=master)](https://travis-ci.org/Metaswitch/sendpacket) [![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT) [![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-green.svg)](http://www.apache.org/licenses/LICENSE-2.0)

`sendpacket` is a high-level rust library for low-level networking that makes use of macros to provide a "kwargs-like" interface a-la python's dpkt/scapy.

With `sendpacket` you can construct and modify arbitrary packet data and attempt to send it via a NIC, which uses `libpnet` under the covers.

`sendpacket` should perform well, but is otherwise fundamentally a hackathon project: it exposes a somewhat hacky API, provides an incomplete set of functionality for general use, and the authors don't promise any level of maintenance.

## Documentation

https://docs.rs/sendpacket

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
