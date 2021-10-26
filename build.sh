#!/usr/bin/env bash
set -e

cross build --release --target x86_64-unknown-linux-musl "${@}"
strip "./target/x86_64-unknown-linux-musl/release/lambda-aws-sso-google-sync"
cp "./target/x86_64-unknown-linux-musl/release/lambda-aws-sso-google-sync" ./target/x86_64-unknown-linux-musl/release/bootstrap 
zip -r9 -j "./lambda-aws-sso-google-sync.zip" ./target/x86_64-unknown-linux-musl/release/bootstrap
