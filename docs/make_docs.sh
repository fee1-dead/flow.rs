#!/bin/bash

set -e

git apply rename.diff

cargo doc -Z unstable-options -Z rustdoc-scrape-examples=all || cargo doc -Z unstable-options -Z rustdoc-scrape-examples=all

git apply -R rename.diff