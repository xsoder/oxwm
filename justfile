build:
    cargo build --release

install: build
    cp target/release/oxwm /usr/bin/oxwm
    chmod +x /usr/bin/oxwm
    @echo "✓ oxwm installed to /usr/bin/oxwm"
    @echo "  Run 'oxwm --init' to create your config"

uninstall:
    rm -f /usr/bin/oxwm
    @echo "✓ oxwm uninstalled"
    @echo "  Your config at ~/.config/oxwm/ is preserved"

clean:
    cargo clean

test-clean:
	pkill Xephyr || true
	rm -rf ~/.config/oxwm
	Xephyr -screen 1280x800 :1 & sleep 1
	DISPLAY=:1 cargo run --release -- --config resources/test-config.lua

test:
	pkill Xephyr || true
	Xephyr -screen 1280x800 :1 & sleep 1
	DISPLAY=:1 cargo run --release -- --config resources/test-config.lua

test-multimon:
	pkill Xephyr || true
	Xephyr +xinerama -screen 640x480 -screen 640x480 :1 & sleep 1
	DISPLAY=:1 cargo run --release -- --config resources/test-config.lua

init:
    cargo run --release -- --init

edit:
    $EDITOR ~/.config/oxwm/config.lua

check:
    cargo clippy -- -W clippy::all
    cargo fmt -- --check

fmt:
    cargo fmt

pre-commit: fmt check build
    @echo "✓ All checks passed!"
