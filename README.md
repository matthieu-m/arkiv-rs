# Arkiv: the Ark of archives!

## What is Arkiv?

Arkiv is a library for reading and writing various archive formats.


## Goals

Functional goals:

 - Correct, including tweaks for supporting popular implementations if they differ from the specifications
 - Complete, both in terms of number of formats and coverage of each format

Non-functional goals:

 - Safe, untrusted archives should be explorable at no risk
 - Portable, from 16-bits to 64-bits platforms, whatever the OS
 - Lightweight, having as little memory or CPU overhead as possible
 - FFI, a C-FFI is exported to facilitate use from other languages


## Non-Goals

Functional non-goals:

 - Extraction-capable, no compression or decompression algorithm provided

Non-functional non-goals:

 - Fastest, reasonably fast is good enough


## Constraints

In order to reach those goals, a few constraints are put forward:

 - Only safe Rust (outside of the ffi module)
 - No internal I/O
 - No internal memory allocation
 - No `panic!`, or panicking code
 - No recursion or loop controlled solely by untrusted bounds

Note: ideally, this code should be `#[no_std]`, at least its non-test portion.


## Architecture

The library exposes a few top-level modules:

 - one high-level uniform module, allowing reading/writing archive formats generically
 - one low-level module per format, allowing access to all the specifities of the format
 - one api module, containing the user-definable traits to operate with the library
 - one ffi module, containing the C FFI modules mirroring each of the above modules

The library also contains a utils module, used to share commonly used code across components.


## Covered formats

Complete:

 - NONE

Partial:

 - NONE

Planned:

 - 7z
 - rar
 - tar
 - zip


## Contributing

The project is in a very early stage, no external contribution is planned yet.
