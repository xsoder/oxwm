build:
    cargo build --release

install: build
    sudo cp target/release/oxwm /usr/local/bin/oxwm
    @echo "✓ oxwm installed to /usr/local/bin/oxwm"
    @echo "  Run 'oxwm --init' to set up your config"

uninstall:
    sudo rm -f /usr/local/bin/oxwm
    @echo "✓ oxwm uninstalled"
    @echo "  Your config at ~/.config/oxwm is preserved"

clean:
    cargo clean

test:
    pkill Xephyr || true
    Xephyr -screen 1280x800 :1 & sleep 1
    DISPLAY=:1 cargo run -- --init
    DISPLAY=:1 ~/.cache/oxwm/oxwm-binary

init:
    cargo run -- --init

recompile:
    cargo run -- --recompile

edit:
    $EDITOR ~/.config/oxwm/config.rs

check:
    cargo clippy -- -W clippy::all
    cargo fmt -- --check

fmt:
    cargo fmt

pre-commit: fmt check build
    @echo "✓ All checks passed!"

