# Binary name as installed on the system.
# Cargo always produces the binary under the package name (hce); this
# variable only controls what name ends up in BINDIR. Override with:
#   make BINARY=hyprland-config-editor install
BINARY  ?= hce

# Installation prefix. The binary goes into $(PREFIX)/bin unless BINDIR is
# overridden independently. Override with:
#   make PREFIX=~/.local install
#   make PREFIX=/usr install
PREFIX  ?= /usr/local

# Binary installation directory. Defaults to $(PREFIX)/bin.
# Override independently when the prefix layout does not follow the standard:
#   make BINDIR=/opt/bin install
BINDIR  ?= $(PREFIX)/bin

# Staging root for package-manager-style installs. Files are written to
# $(DESTDIR)$(BINDIR)/... but the embedded paths stay $(BINDIR)/...
# Example: make DESTDIR=/tmp/pkg PREFIX=/usr install
DESTDIR ?=

# Cargo always names its release output after the [package] name in
# Cargo.toml. Keep this in sync if the package is ever renamed.
CARGO_BIN := hce
CARGO_OUT := target/release/$(CARGO_BIN)

.PHONY: all build install uninstall clean help

all: build

build:
	cargo build --release

# Copy the built binary into place, renaming it to $(BINARY) if needed.
# install(1) creates the directory, sets mode 0755, and is atomic.
install: build
	install -d "$(DESTDIR)$(BINDIR)"
	install -m 0755 "$(CARGO_OUT)" "$(DESTDIR)$(BINDIR)/$(BINARY)"

uninstall:
	rm -f "$(DESTDIR)$(BINDIR)/$(BINARY)"

clean:
	cargo clean

help:
	@printf 'Usage: make [TARGET] [VARIABLE=value ...]\n'
	@printf '\n'
	@printf 'Targets:\n'
	@printf '  all (default)  Build the release binary\n'
	@printf '  build          Run cargo build --release\n'
	@printf '  install        Install binary to DESTDIR+BINDIR\n'
	@printf '  uninstall      Remove the installed binary\n'
	@printf '  clean          Run cargo clean\n'
	@printf '  help           Show this message\n'
	@printf '\n'
	@printf 'Variables (all optional, override on the command line):\n'
	@printf '  BINARY=<name>   Installed binary name   (default: hce)\n'
	@printf '  PREFIX=<path>   Installation prefix     (default: /usr/local)\n'
	@printf '  BINDIR=<path>   Binary directory        (default: PREFIX/bin)\n'
	@printf '  DESTDIR=<path>  Staging root            (default: empty)\n'
	@printf '\n'
	@printf 'Examples:\n'
	@printf '  make install\n'
	@printf '  make install PREFIX=~/.local\n'
	@printf '  make install BINARY=hyprland-config-editor\n'
	@printf '  make install DESTDIR=/tmp/pkg PREFIX=/usr\n'
	@printf '  make uninstall BINARY=hyprland-config-editor PREFIX=~/.local\n'
