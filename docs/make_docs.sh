#!/bin/bash

git apply rename.diff

cargo doc -p flow_sdk -Z unstable-options -Z rustdoc-scrape-examples=examples

git apply -R rename.diff