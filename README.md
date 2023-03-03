<!-- Diagram https://mermaid.js.org/intro/ -->

<!-- Markdown https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax -->

![brand logo](./imgs/brand-logo.svg)

# TMS – Time series compression library

Fast access to large amounts of financial time series data in a single modern language.

- [TMS – Time series compression library](#tms--time-series-compression-library)
  - [Summary](#summary)
  - [Project state](#project-state)
  - [Rust modules, struct, and methods](#rust-modules-struct-and-methods)
  - [Explaining the design](#explaining-the-design)
  - [About compression](#about-compression)
  - [Differential encoding compression](#differential-encoding-compression)
  - [Binary packing compression](#binary-packing-compression)
  - [Variable integer compression](#variable-integer-compression)
  - [SIMD instructions](#simd-instructions)
  - [The `tms` compression algorithm](#the-tms-compression-algorithm)
    - [Compressing a day of NaiveDateTimes to u32s](#compressing-a-day-of-naivedatetimes-to-u32s)
    - [Segmenting a list of u32s into blocks](#segmenting-a-list-of-u32s-into-blocks)
    - [Differential encoding a block with SIMD](#differential-encoding-a-block-with-simd)
    - [Binary packing a block with SIMD](#binary-packing-a-block-with-simd)
    - [Variable integer compression](#variable-integer-compression-1)
    - [Organizing compressed bytes](#organizing-compressed-bytes)
  - [Three-character naming](#three-character-naming)
  - [Figure style and tool](#figure-style-and-tool)

## Summary

`tms` is a time series compression library written in Rust.
* Designed for financial data.
* Optimized for fast decompression with SIMD instructions.
* Good space compression.
* Can be used with in-memory caches.
* Can be used with on-disk storage.

`tms` is an acronym for "Time Series".

`tms` is based on the work described in: 
* "[Decoding billions of integers per second through vectorization](https://arxiv.org/abs/1209.2137)" by Daniel Lemire and Leonid Boytsov.
* "[SIMD Compression and the Intersection of Sorted Integers](https://arxiv.org/abs/1401.6399)" by Daniel Lemire, Leonid Boytsov, and Nathan Kurz.

Compressed time series enables traversing large datasets quickly.

## Project state

`tms` is a work in progress. Some tests work. Some bugs need to be resolved. Some features are to be built. 

The intent to offer existing work, and not wait until all details are resolved. I appreciate when others do the same. I learn from people who are sharing their progress, and hope you will too.

256-bit SIMD vectors on x86 chips are currently supported.

Project tasks:
- [x] Date-time series compression and decompression
- [x] 256-bit SIMD instructions
- [x] Varint compression and decompression
- [x] Some unit tests
- [ ] Evaluate floating point compression approaches
- [ ] Implement a floating point compression approach
- [ ] Refine public API functions for client usability
- [ ] Extensive unit tests
- [ ] Publish to the [crates.io](https://crates.io/) registry

## Rust modules, struct, and methods

The `tms` crate is composed of the `mcr` and `tms` modules. 

![crates](./imgs/crates.svg)
> Figure 1. Modules in the `tms` crate.

The `mcr` module, an acronym for "macro", provides procedural macros which generate compression functions. `mcr` generates `tms`.

The `tms` module, an acronym for "time series", provides generated compression functions. `tms` is meant for use by client libraries. Tests are in the `tms` module, and are manually written. 

The `TmeMli` struct, an acronym for "Time Millisecond", represents a sequence of date-times with millisecond precision. `TmeMli` compresses date-times for multiple days.

Values are compressed by day with the `append_day` method. Values are accessible by day with the `get_day` method. `append_day` accepts an uncompressed list of [NaiveDateTimes](https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDateTime.html). `get_day` returns the uncompressed list of NaiveDateTimes. Internally, NaiveDateTimes are stored as a list of compressed bytes.

## Explaining the design

Different types of compression are explained and then combined into a final explanation for the `tms` compression algorithm.

## About compression

Compression can be explained with the analogy of a box with a gift in it. The box is a data type, and the gift is the actual data that's used.

![Gift in a box](./imgs/one-gift.svg)
> Figure 2. An analogy of a data type as a box, and the gift in the box as the data that's used.

The gift in the box is what we want, but computer chips are physically etched to work with convenient-sized boxes. The box is convenient. It works. And we carry it around.

The gift in the box is so useful, we decide to do more things with more gifts.

![Three gifts, three boxes](./imgs/three-gifts-three-boxes.svg)
> Figure 3. Three boxes, each with gifts.

We start to notice that carrying and using more boxes isn't quite as convenient as when we had just one box. We look into each box and see that the gifts don't take up all the space of the boxes. 

Why don't we just use the gifts? 
1. The used data, the gift, might sometimes expand into the whole box.
2. The processor is physically designed to use boxes of a certain size.

We could use smaller boxes, if we have the option. The computer can use different sized boxes. If we understand the context of the data well enough, and know that we never need a large box, we can choose a smaller box. With smaller boxes, it's also possible that space is still left over in each box. Often the size of each box has to be as large as the largest gift.

In the case where we're given boxes from someone else, we may not have the option to choose a smaller box. For example, using a timestamp or 64-bit floating point value.

So another option can be to use one box for three gifts. That would make it easier to carry the gifts around. That's compression.

![Three gifts, one box](./imgs/three-gifts-one-box.svg)
> Figure 4. An analogy of compression as one box with three gifts.

To have three gifts in one box, each gift has to be moved from their original boxes. It takes time to move the gifts for the convenience of carrying one box. 

Inside the computer processor, to use each gift, we have to move the gifts back into individual boxes before using them. It's effort to program. And it's effort for the processor to move gifts.

![Compression process](./imgs/compression-process.svg)
> Figure 5. An analogy of compression as moving gifts between individual boxes and one box.

So, is it worth compressing data with having to move gifts around? It depends on the context and data. For time series, the answer is yes. The changes between points of a time series are often small enough to enable compression in a reasonble amount of time.

## Differential encoding compression

Differential encoding subtracts one point from another. Instead of remembering `1,000,000`, and `1,000,003`, we just remember `3`. We repeat that for pairs of points. It's easier for both humans and computers to just remember `3`. The requirement is that data be some sequential integers. The two points might be close neighbors; or, one point may be some distant starting point. An unmodified starting point is preserved to enable future decompression.

![Differential encoding](./imgs/differential-encoding.svg)

> Figure X. Differential encoding subtracts one point from another.

## Binary packing compression

Binary packing is moving gifts next to one another in a single box. The gifts can be any size, and aren't necessarily the pre-defined sizes the computer likes to work with. The destination size can be the size of the largest gift.

![Binary packing](./imgs/binary-packing.svg)

> Figure X. An analogy of binary packing compression as placing gifts next to each other in a box.

Binary packing uses bit-shift operations to move the gifts in and out of boxes. Bit-shifting is fast for a computer.

## Variable integer compression

[Variable integer compression](https://en.wikipedia.org/wiki/Variable-length_quantity), also known as `varint`, moves a gift to a box that has a pre-defined size that's just big enough. The computer usually works with pre-defined box sizes of `1`,`2`,`4`, and `8` bytes. If an integer uses one byte within a four-byte box, preserve just one byte.

![Converting NaiveDateTimes to u32s](./imgs/varint.svg)

> Figure X. An analogy of variable integer compression as moving a gift to a smaller box with a pre-defined size.

`varint` is relatively short to program. Ten lines of code are used in the `tms` compress function `usize_pck`.

`varint` is used in the [gRPC](https://grpc.io/) remote procedure framework.

## SIMD instructions

[SIMD instructions](https://en.wikipedia.org/wiki/Single_instruction%2C_multiple_data) make data processing fast. SIMD means "single instruction, multiple data". One processor instruction does one thing on mutliple pieces of data, instead of doing one thing on one piece of data. 

SIMD programming takes additional effort, and can be applied to differential encoding and binary packing.

## The `tms` compression algorithm

The intent is to compress a series of date-times into a small size, while preferring fast decompression. NaiveDateTimes are shaped into forms which are readily used by SIMD instructions.

Times are limited between the hours of `9:30am` to `4:00pm`, which corresponds to a [NYSE Core Trading Session](https://www.nyse.com/markets/hours-calendars). This choice provides improved compression size, and supports SIMD use with u32s. The 6.5 hour day limits a maximum millisecond, and maximum quantity of milliseconds to 23,400,000.

> `1,000` milliseconds * `60` seconds * `60` Minutes * `6.5` hours = `23,400,000` milliseconds per day

25 bits can represent any millisecond within the 6.5 hour range, `2^25 = 33,554,432`. In practice, 4-byte unsigned integers are used during intermediate compression steps, `2^32 = 4,294,967,296`.

### Compressing a day of NaiveDateTimes to u32s

Differential encoding subtracts the date and 9:30am hour from each date-time.
The internal representation of each time becomes smaller. The list of 12-byte NaiveDateTimes are converted to a list of 4-byte u32s. One 32-bit integer stores the date part, and other 32-bit integers store milliseconds from the start of the day.

![Converting NaiveDateTimes to u32s](./imgs/datetimes-to-u32s.svg)

> Figure X. A list of 12-byte NaiveDateTimes compressed to a list of u32s using differential encoding.

### Segmenting a list of u32s into blocks

The list of u32s is then segmented into blocks to support SIMD instructions. Each block has 256 elements. And the last block may have a variable length, depending on the data.

![Segementing blocks](./imgs/block-segmenting.svg)

> Figure X. A list of u32s, represented as milliseconds, are segmented into blocks of 256 elements. The last block may have a variable number of elements.

### Differential encoding a block with SIMD

Applying another round of differential encoding at the block level improves compression size. The additional step is reasonable with SIMD processing being fast.

A SIMD instruction subtracts one list from another. Each list has eight unsigned integers. The first list of eight integers is subtracted from consecutive lists within the block. The first list of integers is stored uncompressed. Remaining delta values are stored.

![Differential encoding with SIMD](./imgs/simd-subtraction.svg)

> Figure X. Subtracting two lists of unsigned integers with a SIMD instruction. Unsigned integers are represented as milliseconds. Elements are subtracted at corresponding indexes.

While the milliseconds are sequential, they can be randomly distributed. Notice that the differences are not sequential.

### Binary packing a block with SIMD

The differential encodings within a block is scanned for a maximum. The maximum is set as the box size for all values in the block. A list of unsigned integers is are bit-shifted with SIMD instructions.

![Binary packing with SIMD](./imgs/simd-binary-packing.svg)

> Figure X. Moving bits into smaller boxes with SIMD bit-shifting. 

A single box size per block supports SIMDs ability to do one thing on many pieces of data. The box size varies from block-to-block for to improve compression size whlie allowing decompression speed.

### Variable integer compression

Any variable length block at the end of the list is compressed using `varint`. A variable length list may or may not exist, depending on the data. The `varint` compression doesn't use SIMD.

### Organizing compressed bytes

The binary packed bytes representing a day are appended to the end of a buffer with previous days. Various bookkeeping variables track the total number of days, the byte index of each day, and other items.

## Three-character naming

`tms` uses three and four character identifiers for modules (`mcr`, `tms`), structs (`TmeMli`, `Itm`, `BlkItr`), and functions (`gen_blk`, `u32_pck`, `u32x256_unp`).

It's a style with a logic. 

It fits well in the mind, once adjusted to. It composes larger concepts with less cognitive overload for human parsing and reasoning. It's a style seen in C, assembly, and Rust. 

It's a compressed word form that can be expanded, if more detailed explanation is needed. It allows for the accumulation and juggling of many parts more quickly and succintly.

When I read `gen_blk`, internally I'm translating and understand it means "Generate Block". Some writers choose `generate_block`. Equally valid. The style also depends on the type of engineering and audience. For a low-level Rust library using SIMD and bit-shifting, three-character naming is helpful.

## Figure style and tool

Figures were created with the [Material 3](https://m3.material.io/) visual design system, [Figma](https://www.figma.com/), and [M3 design kit](https://material.io/blog/material-3-figma-design-kit). 