#!/usr/bin/env bash

mkdir -p out/
echo "*" > out/.gitignore

echo "Downloading site dat file..."
curl -L# -o out/geosite.dat https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geosite.dat \
    || curl -L# -o out/geosite.dat https://cdn.jsdelivr.net/gh/Loyalsoldier/v2ray-rules-dat@release/geosite.dat

echo "Dumping site dat file..."
# dat-dump from https://github.com/lisoboss/dat-dump
dat-dump --path out/geosite.dat --keys geolocation-\!cn --out deploy/data/domain_proxy_list.txt
dat-dump --path out/geosite.dat --keys category-ads-all --out deploy/data/domain_block_list.txt
