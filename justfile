build:
    cargo build --release

install: build
    cp target/release/oxwm ~/.local/bin/oxwm
    chmod +x ~/.local/bin/oxwm
    @echo "✓ oxwm installed to ~/.local/bin/oxwm"
    @echo "  Run 'oxwm --init' to create your config"

uninstall:
    rm -f ~/.local/bin/oxwm
    @echo "✓ oxwm uninstalled"
    @echo "  Your config at ~/.config/oxwm/config.ron is preserved"

clean:
    cargo clean

test-clean:
	pkill Xephyr || true
	rm -rf ~/.config/oxwm
	Xephyr -screen 1280x800 :1 & sleep 1
	DISPLAY=:1 cargo run --release -- --config resources/test-config.ron

test:
	pkill Xephyr || true
	Xephyr -screen 1280x800 :1 & sleep 1
	DISPLAY=:1 cargo run --release -- --config resources/test-config.ron

test-multimon:
	pkill Xephyr || true
	Xephyr +xinerama -screen 640x480 -screen 640x480 :1 & sleep 1
	DISPLAY=:1 cargo run --release -- --config resources/test-config.ron

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
