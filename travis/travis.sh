#!/bin/bash
set -ev
set -o nounset
set -o pipefail

if [ "${JOB:?}" = "rustfmt" ]; then
    rustfmt --write-mode diff -- **/*.rs
elif [ "${JOB:?}" = "test" ]; then
    cargo build --verbose
    cargo test --verbose
else
    printf "Unknown job: %s\n" "${JOB:?}"
    exit 1
fi
