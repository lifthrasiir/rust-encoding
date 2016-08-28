.PHONY: all
all:
	@echo 'Try `cargo build` instead.'

.PHONY: authors
authors:
	echo 'Encoding is mainly written by Kang Seonghoon <public+rust@mearie.org>,' > AUTHORS.txt
	echo 'and also the following people (in ascending order):' >> AUTHORS.txt
	echo >> AUTHORS.txt
	git log --format='%aN <%aE>' | grep -v 'Kang Seonghoon' | sort -u >> AUTHORS.txt

.PHONY: test
test:
	# `test_correct_table` tests with indices with non-BMP mappings tend to be
	# very slow without the optimization, so japanese and tradchinese got flags
	cargo test -v
	cargo test -v -p encoding-index-singlebyte
	cargo test -v -p encoding-index-korean
	RUSTFLAGS='-C opt-level=1' cargo test -v -p encoding-index-japanese
	cargo test -v -p encoding-index-simpchinese
	RUSTFLAGS='-C opt-level=1' cargo test -v -p encoding-index-tradchinese
	cargo test -v -p encoding-types

.PHONY: readme
readme: README.md

README.md: src/lib.rs
	# really, really sorry for this mess.
	awk '/^\/\/! # Encoding /{print "[Encoding][doc]",$$4}' $< > $@
	awk '/^\/\/! # Encoding /{print "[Encoding][doc]",$$4}' $< | sed 's/./=/g' >> $@
	echo >> $@
	echo '[![Encoding on Travis CI][travis-image]][travis]' >> $@
	echo >> $@
	echo '[travis-image]: https://travis-ci.org/lifthrasiir/rust-encoding.png' >> $@
	echo '[travis]: https://travis-ci.org/lifthrasiir/rust-encoding' >> $@
	awk '/^\/\/! # Encoding /,/^\/\/! ## /' $< | cut -b 5- | grep -v '^#' >> $@
	echo '[Complete Documentation][doc] (stable)' >> $@
	echo >> $@
	echo '[doc]: https://lifthrasiir.github.io/rust-encoding/' >> $@
	echo >> $@
	awk '/^\/\/! ## /,!/^\/\/!/' $< | cut -b 5- >> $@
