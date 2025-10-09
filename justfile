ensure-config:
    @if [ ! -f src/config.rs ]; then \
        echo "Creating config.rs from default_config.rs..."; \
        cp src/default_config.rs src/config.rs; \
        echo "✓ Edit src/config.rs to customize your setup"; \
    fi

build: ensure-config
    cargo build --release

install: build
    sudo cp target/release/oxwm /usr/local/bin/oxwm
    @echo "✓ oxwm installed. Restart X or hit Mod+Shift+R (when implemented)"

uninstall:
    sudo rm -f /usr/local/bin/oxwm

clean:
    cargo clean

test: ensure-config
    pkill Xephyr || true
    Xephyr -screen 1280x800 :1 & sleep 1
    DISPLAY=:1 cargo run

reset-config:
    cp src/default_config.rs src/config.rs
    @echo "✓ Config reset to default"
