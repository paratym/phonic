# Phonic

Phonic is a rust library for digital signal processing with a focus on audio. The api is loosely based on the `std::io` interface and aims to be modular, extensible, and to make as few assumptions about its surrounding systems as possible.

## Features

- Buffers can be statically allocated and uninitialized
- All structures that require an internal buffer are generic over `Borrow<[T]>`
- Flexible blocking interface to allow for yielding time back to an arbitrary scheduler
- Support for both static and dynamic construction of formats and codecs

## Modules

It is recommended to use the `phonic` crate and enable the necessary features instead of the individual crates unless you are implementing a library.

| Module | Crate         | Description                                                |
| ------ | ------------- | ---------------------------------------------------------- |
| \*     | phonic_signal | The core signal traits                                     |
| io     | phonic_io     | Traits for working with audio formats and codecs           |
| dsp    | phonic_dsp    | Utilities for generating, analyzing, and modifying signals |
| sync   | phonic_sync   | Types for synchronizing signals between threads            |
| cpal   | phonic_cpal   | Integration with [cpal](https://github.com/rustaudio/cpal) |
