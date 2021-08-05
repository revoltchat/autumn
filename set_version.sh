#!/bin/bash
export version=1.1.1-patch.0
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
