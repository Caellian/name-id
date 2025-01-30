Unique identity derived from string hashes.

This crate provides means of creating globally unique (per input hash)
identifiers that can be computed at both compile-time and run-time.

It's completely `no_std` compatible and `alloc` can be disabled as well.

## Usage

```rust
use name_id::{NameId, id};

// any ident token will be treated as a string
const IDENT_SINGLE: NameId = id!(some_id_ident);
// multiple tokens will be concatenated into a string with ' ' delimiter
const IDENT_SEQUENCE: NameId = id!(can even be 6 or more);
// string representations will be used in case of literals
const STRING_ID: NameId = id!("id macro supports string values");
// so for numbers, their string representation will be hashed
const NUMBER_ID: NameId = id!(256);
// and any valid utf-8 character can be used
const SPECIAL_ID: NameId = id!("!%$#");

#[allow(unused_assignments)]
fn main() {
    // NameId can be checked for equality against other NameIds
    assert_eq!(IDENT_SINGLE, NameId::new("some_id_ident"));
    // they can also be checked against any AsRef<str>, which will be
    // automatically hashed for comparison using the same hashing algorithm the
    // crate uses
    assert_eq!(IDENT_SEQUENCE, "can even be 6 or more");

    // hash values can be accessed via a const function
    #[cfg(feature = "ahash")]
    assert_eq!(STRING_ID.value(), 10398550419565578837);

    let are_equal = const {
        let mut const_variable = STRING_ID;
        const_variable = NUMBER_ID;

        // there are also const_eq and const_cmp utility functions for checking
        // whether IDs are equal at compile time:
        NUMBER_ID.const_eq(&const_variable)
    };

    if are_equal && SPECIAL_ID == "!%$#" {
        println!("All checks passed.");
    }
}
```

See the docs for [`NameId`] and [`id!`] for details on functionality.

## Features

Functionality of this crate can be tweaked using various features:

- `alloc` (_default_) - enables support for allocation and allows creating
    `NameId` from non-static strings by leaking a copy of their name in debug
    builds (to make it `'static`).
- `detect_collisions` - enables panic on detected collisions of **runtime created**
  `NameId`s.
- `debug_name` - adds ID label for debug builds
- `fixed_size` - adds padding in place of `name: &'static str` for release
  builds so `NameId` size doesn't change between those and debug builds if
  `debug_name` is enabled.
- Hasher features listed in [Supported hashers](#Supported-hashers) section.

### Supported hashers

Any persistent hashers that are `no_std` and no-alloc compatible can be added.

Currently supported hashers are:

| Hasher | Feature |                   Crate                   |
| :----: | :-----: | :---------------------------------------: |
| ahash  | `ahash` | [`ahash`](https://crates.io/crates/ahash) |
