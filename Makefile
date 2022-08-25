# gaufre - gopher client
include config.mk

all: gaufre
gaufre: src/main.rs src/config.rs
	cargo build --release
	cp ./target/release/gaufre ./
src/config.rs: src/config.def.rs
	cp $< $@

clean:
	cargo clean
	rm -f gaufre
install: gaufre
	mkdir -p $(DESTDIR)$(PREFIX)/bin
	cp -f gaufre $(DESTDIR)$(PREFIX)/bin
	chmod 755 $(DESTDIR)$(PREFIX)/bin/gaufre
uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/gaufre

.PHONY: all clean install uninstall
