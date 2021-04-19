# The Shooter
Advanced Operating Systems project made using [Rust](https://www.rust-lang.org/)

### Linux Kernel used
- Linux 5.8.0-50-generic #56~20.04.1-Ubuntu x86_64 GNU/Linux

### How to install rust
Run:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
More info [here](https://www.rust-lang.org/tools/install)
### Requirements to run the code
- Rust toolchain installed:
    - (rustc 1.51.0 (2fd73fabe 2021-03-23))
    - (cargo 1.51.0 (43b129a20 2021-03-16))


### How to run:
1. Inside the root project folder, run:

    Print info of a Fat16 filesystem:
    ```
    cargo run /info Fat16
    ```
    Print info of a Ext2 filesystem:
    ```
    cargo run /info Ext2
    ```
    Print info of an unkown filesystem:
    ```
    cargo run /info non_def
    ```
