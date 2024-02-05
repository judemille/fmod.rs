[![docs.rs](https://img.shields.io/docsrs/fmod-sys)](https://docs.rs/fmod-sys) [![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/judemille/fmod-sys/rust.yml)](https://github.com/judemille/fmod-sys/actions)

# fmod-sys: Rust bindings for the FMOD Engine SDK

This crate provides raw, low-level bindings to the FMOD Engine SDK.

## Disclaimer

The current maintainer of this project is a trans lesbian who unequivocally supports
Ukraine, and opposes any and all human rights violations.

### *You should not use this project if you:*

- Do not unequivocally support the LGBTQ+ population, including transgender
  individuals.
- Think that LGBTQ+ people "shouldn't put it out on display"
- Support "drop the T", TERF, or similar movements.
- Think that pedophilia is included in LGBTQ+, either because you want it to be
  included, or you think that the community accepts it. It does not accept it.
- Refuse to address and refer to people with their preferred name, pronouns, and
  gender labels.
- Do not support Ukraine's struggle against their Russian oppressors.
- Support any far-right parties or politicians (including Vladimir Putin, the GOP,
  AfD, FdI, and similar)

I cannot stop you, but anyone observed to meet the above listed criteria who
interacts with the project will be blocked from interaction.

## License

Licensed under the Mozilla Public License, version 2.0.

### What does this mean for me?

I get it, you don't have time to read the license. Here's some bullet points on what
this license means for you.

- You may combine this library with other work that is under a different license, so
  long as the files of this library remain separate.
  - The code of this library, under the MPL-2.0 license (or compatible), must be made
    readily available to users.
  - Recipients of the larger work must be made aware of the use of this library, its
    license, and how to acquire the code of this library.
- Any modifications of this library's files must be published under the MPL-2.0.
- You may use this library commercially, so long as it is made clear that it is done
  on your own behalf, and not on the behalf of the contributors.

There is some more nuance than that, but those bullet points cover the general points.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you shall be licensed as above, without any additional terms
or conditions.

All commits must be signed-off (`git commit -s`). This is a declaration that you have
read and agree to the terms in the
[Developer Certificate of Origin](https://developercertificate.org/) (DCO). This can
be compared to a CLA, but is different in that your copyright remains with you. All
you are doing is attesting that you can contribute code under the repository's license.
A copy of the DCO text is kept in this repository at DCO.txt.

You may sign the DCO with your preferred name, so long as it is relatively consistent,
so we can keep track of who's who.

## Minimum Supported Rust Version

The MSRV of this crate is always the latest stable compiler version at the time of a
given commit. Maybe it'll work on an older version. Maybe it won't. No guarantees.

This will probably change once a stable release goes out.

## Unit Testing

The `mockall` feature of this crate is intended to be enabled when unit testing a
crate that uses this crate. All functions will be mocked, using
[mockall.](https://github.com/asomers/mockall) This crate currently exposes
`mockall = "~0.12"`.

Functions live in the `functions` module, which, when mocking is enabled, is doubled
up as `mock_functions`. It can be used like this:

``` rust
#[cfg(not(test))]
use fmod_sys::functions as fmod_sys_fns;
#[cfg(test)]
use fmod_sys::mock_functions as fmod_sys_fns;

use fmod_sys_fns::{...};
```

You can also use the `mockall::double` macro if you so desire.

