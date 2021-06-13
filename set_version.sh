#!/bin/bash
export version=1.1.0
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
