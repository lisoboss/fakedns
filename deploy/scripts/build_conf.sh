#!/usr/bin/env bash

set -o pipefail
build() {
    cat $1 | rg -v '^regexp:' | sed 's/^full://' | rev | sort | uniq | rev | python deploy/scripts/process_domains.py | sort | uniq > $2
    return $?
}

set -e
build "deploy/custom/conf.d/domain.conf deploy/data/domain_proxy_list.txt" deploy/conf.d/domain.conf
build "deploy/custom/conf.d/domain_block.conf deploy/data/domain_block_list.txt" deploy/conf.d/domain_block.conf
build deploy/custom/conf.d/domain_exclude.conf deploy/conf.d/domain_exclude.conf
