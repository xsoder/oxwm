build:
    cargo build --release

install: build
    mkdir -p /usr/local/bin/
    mkdir -p /usr/share/xsessions/
    install -Dm755 oxwm /usr/local/bin
    install -Dm644 oxwm.desktop /usr/share/xsessions
    @echo "✓ oxwm installed to ~/.local/bin/oxwm"
    @echo "✓ oxwm-desktop session added to /usr/share/xsessions/oxwm.desktop"

uninstall:
    rm -f ~/.local/bin/oxwm
    rm -f /usr/share/xsessions/oxwm.desktop 
    @echo "✓ oxwm uninstalled"
    @echo "✓ oxwm.desktop session uninstalled"
    @echo "  Your config at ~/.config/oxwm/config.ron is preserved"

clean:
    cargo clean

test-clean:
	pkill Xephyr || true
	rm -rf ~/.config/oxwm
	Xephyr -screen 1280x800 :1 & sleep 1
	DISPLAY=:1 cargo run --release

test:
	pkill Xephyr || true
	Xephyr -screen 1280x800 :1 & sleep 1
	DISPLAY=:1 cargo run --release

init:
    cargo run --release -- --init

edit:
    $EDITOR ~/.config/oxwm/config.ron

check:
    cargo clippy -- -W clippy::all
    cargo fmt -- --check

fmt:
    cargo fmt

pre-commit: fmt check build
    @echo "✓ All checks passed!"
