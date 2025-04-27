#!/bin/bash
set -e

TARGET=$1

if [ "$TARGET" == "MT" ]; then
    export PATH="$PATH:/home/do30032003/toolchain/MT7688_OW19/staging_dir/toolchain-mipsel_24kc_gcc-8.3.0_musl/bin"
    cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release --verbose --target mipsel-unknown-linux-musl --no-default-features
    upx target/mipsel-unknown-linux-musl/release/ToolCheckConfig
    echo -e "\033[32m✅ Build for MT finished!\033[0m"

elif [ "$TARGET" == "AI" ]; then
    export PATH="$PATH:/home/do30032003/toolchain/AI/staging_dir/gcc-linaro-7.5.0-2019.12-x86_64_aarch64-linux-gnu/bin"
    cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release --verbose --target aarch64-unknown-linux-gnu --no-default-features
    upx target/aarch64-unknown-linux-gnu/release/ToolCheckConfig
    echo -e "\033[32m✅ Build for AI finished!\033[0m"

else
    echo -e "\033[31m❌ Unknown target: $TARGET\033[0m"
    echo "Usage: ./build.sh [MT|AI]"
    exit 1
fi
