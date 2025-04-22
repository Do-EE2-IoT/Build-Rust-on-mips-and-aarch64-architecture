export PATH=$PATH:/home/do30032003/toolchain/AI/staging_dir/gcc-linaro-7.5.0-2019.12-x86_64_aarch64-linux-gnu/bin
cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release --verbose --target aarch64-unknown-linux-gnu --no-default-features
upx target/mipsel-unknown-linux-musl/release/test_project

