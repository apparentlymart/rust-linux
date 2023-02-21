# rust-linux

This is a collection of Rust crates providing various levels of abstraction
over the Linux system call ABI.

[`linux-unsafe`](https://docs.rs/linux-unsafe/) is the lowest level crate and
exposes both a direct interface to making arbitrary system calls and low-level
wrapper functions for many system calls.

[`linux-io`](https://docs.rs/linux-io/) wraps I/O-related system calls in
minimal safe abstractions while avoiding adding unnecessary overhead. The goal
of this crate is to achieve good coverage of the various APIs that work with
files and file descriptors, while also acting as an intermediate layer for
higher-level abstractions for particular kinds of file descriptor.

Over time there will hopefully be various higher-level wrappers around the
two main crates that provide safe wrappers around the various `ioctl`-based
(or similar) device driver APIs. Currently that includes:

- [`linux-kvm`](https://docs.rs/linux-kvm/): provides `ioctl` request constants
  for the KVM subsystem, and wrapper types to make working with the KVM API
  safer and more convenient.

Crates from outside this repository can also optionally use the
`linux-unsafe` and/or `linux-io` API to implement similar wrappers without
having to reimplement the lower-level system call interfaces.

## Contributing

For `linux-unsafe` the goal is to over time provide direct wrappers for all
reasonable system calls. If you need support for a new call that doesn't yet
have a wrapper and the system call ABI doesn't have any "tricky" characteristics
that can't be directly represented in Rust, like variable numbers of arguments,
a PR is welcome to add it and any new types it relies on! Try to follow the
signatures documented in man section 2 as long as they directly describe the
system call parameters.

Some system calls are trickier to map to Rust either because they use C features
that don't translate well or because they are normally used by wrappers in libc
and have quite a different API at the system call level. For these I'd appreciate
a discussion in an issue first to see what might make sense as a minimal Rust
abstraction.

The `linux-io` crate's design is still evolving but broadly the idea is that
its `File` struct type should have _some_ form of each system call which
operates on at least one file descriptor. The ones which take only one file
descriptor as the first argument are the easiest to map. If the underlying
system call involves raw pointers or other unsafe fodder then we start with an
unsafe wrapper around the raw system call and consider higher-level abstractions
wrapping that. If a particular system call seems to require "programming with
types" to safely wrap it, please start a discussion in an issue first to settle
on a suitable abstraction.

I assume that all contributions are offered under the terms of the same MIT
license that this library uses, unless otherwise stated. I'm unlikely to accept
contributions under other licenses because I want to keep the licensing
situation for this repository relatively simple.

If all else fails, you can use `linux-unsafe`'s system call numbers to create
your own wrappers around arbitrary system calls and build upwards from there.
A key goal of the design of this crate is to expose all of the underlying
features that each new abstraction builds on, so you can integrate at whatever
level suits your needs. Feel free to do that if my design goals and contribution
guidelines are not a good fit for your goals!
