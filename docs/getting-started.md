# Getting Started

## Installation

```sh
git clone https://github.com/SeungYeop/astron
cd astron
cargo build --release
```

The binary is at `target/release/astron`.

## Running a Program

```sh
astron <file.astrn>
astron <file.astrn> --launch <name>
astron --help
astron --version
```

`--launch` selects which `launch` block to run. Defaults to `main`.

## Hello World

```astrn
launch main {
    fire log("hello, world")
    land 0
}
```

Every `launch` block must end with `land <int>`. The default entry point is `launch main`.
