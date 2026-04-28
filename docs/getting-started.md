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
```

## Hello World

```astrn
launch main {
    fire log("hello, world")
    land 0
}
```

Every Astron program requires a `launch main` block as its entry point and must end with `land <int>`.
