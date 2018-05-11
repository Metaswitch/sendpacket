# sendpacket

`sendpacket` is a high-level rust library for low-level networking that makes use of macros to provide a "kwargs-like" interface a-la python's dpkt/scapy.

With `sendpacket` you can construct and modify arbitrary packet data and attempt to send it via a NIC, which uses `libpnet` under the covers.

`sendpacket` should perform well, but is otherwise fundamentally a hackathon project: it exposes a somewhat hacky API, provides an incomplete set of functionality for general use, and the authors don't promise any level of maintenance.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
