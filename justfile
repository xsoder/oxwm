build:
    cargo build --release

install: build
    sudo cp target/release/oxwm /usr/local/bin/oxwm
    @echo "✓ oxwm installed. Restart X or hit Mod+Shift+R (when implemented)"

uninstall:
    sudo rm -f /usr/local/bin/oxwm

clean:
    cargo clean

test:
    pkill Xephyr || true
    Xephyr -screen 1280x800 :1 & sleep 1
    DISPLAY=:1 cargo run
