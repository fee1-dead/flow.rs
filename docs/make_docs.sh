#!/bin/bash

set -e

git apply rename.diff

cargo doc -p flow_sdk -Z unstable-options -Z rustdoc-scrape-examples=all

git apply -R rename.diff