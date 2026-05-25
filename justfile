default:
    @just --list

install:
    cargo install --path .

clean:
    cargo clean

build:
    cargo build --release

format:
    bunx prettier --write website/
