# Run oxwm inside Xephyr with some test clients
test:
    pkill Xephyr || true
    Xephyr -screen 1280x800 :1 & sleep 1
    DISPLAY=:1 xterm &
    DISPLAY=:1 xclock &
    DISPLAY=:1 cargo run

