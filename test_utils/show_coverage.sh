#!/bin/sh
xvfb-run cargo llvm-cov --open --ignore-filename-regex '(x11|ewmh_manager|config|keyboard)\.rs$'
