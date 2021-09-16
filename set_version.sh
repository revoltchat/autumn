#!/bin/bash
export version=1.1.2
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
