# leprd

A toy Java Virtual Machine written in Rust.

`leprd` is in very early development. Notably, it is lacking:
- Exceptions
- Just-In-Time compilation
- JNI
- Synchronization
- (Some) Reflection

`leprd` depends on the openjdk 22 implementation of the standard library. In an existing openjdk installation, one can find this in the `libs` directory.

On nixos:
```sh
mkdir modules && cd modules
cp $(sudo nix eval --raw nixpkgs#jdk22.outPath)/lib/openjdk/lib/modules .
jimage extract modules
```
