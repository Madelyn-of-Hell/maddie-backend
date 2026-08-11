cargo build --release --target=x86_64-unknown-linux-gnu

scp target/x86_64-unknown-linux-gnu/release/maddie_backend root@oneluckymushroom.dev:~/site
scp -r maddie_website root@oneluckymushroom.dev:~/site