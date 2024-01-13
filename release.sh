#!/bin/sh

INCREMENT=${INCREMENT:-patch}

cargo bump $INCREMENT
cargo check # To update Cargo.lock

NEW_VERSION=`cargo pkgid | cut -d# -f2 | cut -d: -f2`
echo $NEW_VERSION

git add Cargo.toml Cargo.lock
git commit -m "release: ${NEW_VERSION}"
git tag ${NEW_VERSION}
git push