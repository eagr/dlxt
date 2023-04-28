# dlxt

ergonomic download & extract

```toml
[dependencies]
dlxt = { git = "https://github.com/eagr/dlxt.git", branch = "master" }
```

## Features

- [x] parallel downloads
- [x] auto extraction by extension
    * `.tar`
    * `.bz2` `.gz` `.xz`
- [ ] async downloads

**Download and extract MNIST**

```rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    dlxt::dlxt_sync(
        &[
            "http://yann.lecun.com/exdb/mnist/train-images-idx3-ubyte.gz",
            "http://yann.lecun.com/exdb/mnist/train-labels-idx1-ubyte.gz",
            "http://yann.lecun.com/exdb/mnist/t10k-images-idx3-ubyte.gz",
            "http://yann.lecun.com/exdb/mnist/t10k-labels-idx1-ubyte.gz",
        ],
        "./mnist"
    )?;

    Ok(())
}

// ./mnist/t10k-images-idx3-ubyte
// ./mnist/t10k-labels-idx1-ubyte
// ./mnist/train-images-idx3-ubyte
// ./mnist/train-labels-idx1-ubyte
```

**Download and extract 7z**

```rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    dlxt::dlxt_sync(
        &["https://www.7-zip.org/a/7z2201-linux-x64.tar.xz"],
        "./7z",
    )?;

    Ok(())
}
```

## License

Licensed under either [MIT](/LICENSE-MIT) or [Apache License 2.0](/LICENSE-APACHE) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
