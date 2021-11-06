#!/bin/bash

set -e

git apply rename.diff

cargo doc -Z unstable-options -Z rustdoc-scrape-examples=examples

git apply -R rename.diff