build:
    cargo build --release

install: build
    cp target/release/oxwm ~/.local/bin/oxwm
    chmod +x ~/.local/bin/oxwm
    @echo "✓ oxwm installed to ~/.local/bin/oxwm"
    @echo "  Run 'oxwm --init' to set up your config"

uninstall:
    rm -f ~/.local/bin/oxwm
    @echo "✓ oxwm uninstalled"
    @echo "  Your config at ~/.config/oxwm is preserved"

clean:
    cargo clean

test:
    pkill Xephyr || true
    rm -rf ~/.cache/oxwm  
    Xephyr -screen 1280x800 :1 & sleep 1
    DISPLAY=:1 cargo run --release  

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
