#!/bin/bash

download_dir="downloads"

# Downloading required fonts
mkdir -p $download_dir/fonts/roboto
if [ ! -f "$download_dir/fonts/roboto/Roboto-Medium.ttf" ]; then 
    wget -nv -O $download_dir/fonts/roboto.zip https://fonts.google.com/download?family=Roboto
    unzip -q -o $download_dir/fonts/roboto.zip -d $download_dir/fonts/roboto
    rm $download_dir/fonts/roboto.zip
fi

# Download the pre-defined brotli dictionary
mkdir -p $download_dir/brotli
if [ ! -f "$download_dir/brotli/dictionary" ]; then
    wget -nv -O $download_dir/brotli/dictionary https://github.com/google/brotli/raw/master/c/common/dictionary.bin
fi

# Download brotli test files
mkdir -p $download_dir/brotli/testdata
if [ ! -f "$download_dir/brotli/testdata/tests" ]; then
    # # do a sparse checkout, we don't need the entire repository
    git clone \
        -q \
        --filter=blob:none  \
        --sparse \
        https://github.com/google/brotli \
        "$download_dir/brotli/testdata"

    git -C "$download_dir/brotli/testdata" sparse-checkout set tests/testdata
fi
