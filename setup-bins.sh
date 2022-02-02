#!/bin/sh

set -euxC

mkdir -p bin
for version in $(git tag); do
	git checkout "$version"
	cargo build --release
	mv target/release/pyseud2eq bin/pyseud2eq-$version
	# hacky workaround for lockfile versioning issues
	rm Cargo.lock
done

git checkout master
git checkout master Cargo.lock
