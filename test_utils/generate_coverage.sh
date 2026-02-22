#!/bin/sh
xvfb-run cargo llvm-cov --lcov --output-path ./target/lcov.info --ignore-filename-regex '(x11|ewmh_manager|config|keyboard)\.rs$'
