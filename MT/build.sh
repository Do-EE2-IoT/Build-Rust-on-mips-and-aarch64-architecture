export PATH=$PATH:/home/do30032003/toolchain/MT7688_OW19/staging_dir/toolchain-mipsel_24kc_gcc-8.3.0_musl/bin
cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release --verbose --target mipsel-unknown-linux-musl --no-default-features
