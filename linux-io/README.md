# `linux-io` crate for Rust

This crate offers some lightweight wrappers around direct Linux system calls
related to file descriptors.

The goal is to make it convenient to work with the Linux system call interface
and bypass `std`/libc without introducing any unnecessary additional
abstractions beyond slight transformations of argument types and return values.

```rust
let mut f = File::create_raw(filename, 0o666)?
f.write_all(message)?; // (using the std::io::Write trait)
f.close()?;
```

By default this crate implements traits from the standard library where they
make sense, but you can disable the default crate feature named `std` to
remove those trait implementations and thereby make this crate usable in
`no_std` environments.
