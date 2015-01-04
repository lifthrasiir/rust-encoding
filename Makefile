.PHONY: test
test:
	cargo test -v
	cargo test -v -p encoding-index-singlebyte
	cargo test -v -p encoding-index-korean
	cargo test -v -p encoding-index-japanese
	cargo test -v -p encoding-index-simpchinese
	cargo test -v -p encoding-index-tradchinese
