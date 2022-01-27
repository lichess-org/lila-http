#!/bin/sh -e
cargo +stable build --release --target x86_64-unknown-linux-musl
ssh "root@$1.lichess.ovh" mv /usr/local/bin/lila-http /usr/local/bin/lila-http.bak || (echo "first deploy on this server? set up service and comment out this line" && false)
scp ./target/x86_64-unknown-linux-musl/release/lila-http "root@$1.lichess.ovh":/usr/local/bin/lila-http
ssh "root@$1.lichess.ovh" systemctl restart lila-http
