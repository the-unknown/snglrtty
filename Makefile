BINARY  = snglrtty
PREFIX  ?= $(HOME)/.local
BINDIR  = $(PREFIX)/bin

.PHONY: all setup install uninstall clean

all:
	cargo build --release

setup:
	cp scripts/commit-msg .git/hooks/commit-msg
	chmod +x .git/hooks/commit-msg

install: all
	install -Dm755 target/release/$(BINARY) $(BINDIR)/$(BINARY)

uninstall:
	rm -f $(BINDIR)/$(BINARY)

clean:
	cargo clean
