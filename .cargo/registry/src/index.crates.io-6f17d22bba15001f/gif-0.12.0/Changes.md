# v0.12.0

Features:
- Add compression of pre-compressed frame data, via `Encoder::write_lzw_pre_encoded_frame`.
- The `color_quant` dependency is now optional. Turning it off disables some
  interfaces that would internally build quantization tables. The generic
  implementation of creating such tables can be prohibitively costly compared
  to specialized algorithms in some use cases.

Optimization:
- Avoid some allocations in by replacing `flat_map` argument with arrays

# v0.11.4

Bufixes:
- Fix decoding confusing superfluous image data from previous frames with
  current frame data.
- Bump minimum required version of `weezl`.

Features:
- Add `Encoder::{get_ref, get_mut, into_inner}` to access underlying stream.

# v0.11.3

Bugfixes:
- Fix panic while decoding some images, has no precise cause in the file.
- Warn about `set_extensions` being unimplemented...

Features:
- Added `StreamingDecoder::version` to query the precise version of the
  standard used for encoding the file. This is merely a hint.
- Added `DecodeOptions::allow_unknown_blocks` to skip over unknown or
  unspecified block kinds.

Optimization:
- `Frame::from_rgba` now recognizes when less than 256 colors are being used,
  dynamically skipping the quantization phase.
- Encoding image chunks is faster and simpler 


# v0.11.2

- Fix panic when LZW code size is invalid
- Added option to omit check for lzw end code

# v0.11.1

- Frames out-of-bounds of the screen descriptor are again accepted by default.
- Added `DecodeOptions::check_frame_consistency` to turn this validation on.

# v0.11

- Rename `Reader` to `Decoder`.
- Reworked `Decoder` into `DecodeOptions`.
- The decoding error is now opaque and no longer allocates a string. Adding
  more information or more error conditions is forward compatible.
- Replace the lzw decoder with `weezl`, up to +350% throughput.
- The dysfunctional C-API has been (temporarily?) removed
  - It may get reintroduced as a separate crate at some point
- Added a `std` feature. It must be active for now.
