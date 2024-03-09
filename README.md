# Syphon

A set of types and utilities for audio processing written in pure rust. The main module `signal` defines types for building and describing pcm signals as readers/writers. It is assumed that all signals have a constant sample rate and interleaved channels. the other two: `io` and `dsp` both depend on `signal`. `io` provides audio formatters and codecs also as readers/writers, and methods for dynamically resolving/constructing them. `dsp` (which currently consists of a sine generator) will eventually define types for creating signal chains to generate and manipulate digital signals.

## Disclaimer

Not on crates.io yet because I need a new name

This project is very early in development. It's untested, and the api changes with every commit. please dont use it... yet. If you are looking for something to use in your own project and somehow found this page before any of these, check them out:

- [Symphonia](https://github.com/pdeljanov/Symphonia)
- [dasp](https://github.com/RustAudio/dasp)
- [hound](https://github.com/ruuda/hound)

## Roadmap

[x] Extensible "known" formats/codecs \
[ ] FFMPEG integration \
[ ] Write format resolver for byte string identifiers \
[ ] Finish the sample conversions \
[ ] Complete the core signal adapters \
[x] Feature flagging \
[x] Cpal integration \
[x] Seeking support \
[ ] Logging \
[ ] Real time signal wrapper \
[ ] Benchmarks \
[ ] Sample buffer pool \
