all: target/release/bitbot-rs install

install:
	cp target/release/bitbot-rs ${HOME}/.local/bin/bb
	cp conf.toml ${HOME}/.local/bin/conf.toml

target/release/bitbot-rs: src/main.rs
	cargo build --release
