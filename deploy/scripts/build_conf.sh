#!/usr/bin/env bash

build=target/release/build_domains
if [ ! -f "$build" ]; then
    cargo build --release --bin build_domains
fi

set -e

echo "Building domains..."
$build -o deploy/conf.d/domain.conf \
    -i deploy/custom/conf.d/domain.conf deploy/data/domain_proxy_list.txt 

echo "Building block domains..."
$build -o deploy/conf.d/domain_block.conf \
    -i deploy/custom/conf.d/domain_block.conf deploy/data/domain_block_list.txt

echo "Building exclude domains..."
$build -o deploy/conf.d/domain_exclude.conf \
    -i deploy/custom/conf.d/domain_exclude.conf
