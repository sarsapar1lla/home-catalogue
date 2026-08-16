# HomeCat

A multi-media home catalogue for keeping track of our stuff!

## Local Development

Build the project locally using:

```sh
$ cargo build
```

### Cross compiling for Raspberry Pi

I couldn't get this working locally yet so using a GitHub runner instead.

The workflow is triggered on push and compiles the `./homecat` binary for
the target architecture.

This can then be downloaded as a `.zip` and extracted, then transferred
to the RP via ssh:

```sh
$ nix-shell -p unzip
...
$ unzip ./home-catalogue
$ scp ./home-catalogue homecat@<ip-address>:~/homecat
```
