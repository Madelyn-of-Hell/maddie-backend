CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc cargo build --release --target=x86_64-unknown-linux-gnu

sh push-binary.sh
sh push-site.sh