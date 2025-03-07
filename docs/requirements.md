# Requirements

RustNES is a NES emulator that:
* is built following a test-first approach
* is performant
* is full-featured
* is capable to run all NES games, including those that use special or hidden features of the original NES architecture
* runs on desktop and web (using WebAssembly)
* it's fully debuggable
* it shows results since the beginning


# Coding standards
* use well-known crates (e.g. serde, anyhow, thiserror, bitflags, ...) when needed, to avoid reinventing the wheel. 