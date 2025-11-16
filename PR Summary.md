# PR Summary

## Fix bar flickering and improve configuration management

This release addresses several critical bugs and enhances the user experience:

- Implement double-buffered rendering for the status bar to eliminate flickering
- Restructure LSP configuration with automatic lib/ directory management and .luarc.json setup
- Refactor Lua API error handling, replacing verbose map_err chains with idiomatic ? operator
- Add monocle layout for maximized single-window view
- Consolidate fullscreen actions and improve floating window behavior
- Update test configuration with better documentation and cleaner bar block API
- Bump version to 0.7.1

The bar rendering now uses XPixmap for off-screen composition before copying to the window,
significantly reducing visual artifacts during updates. Configuration updates are now applied
automatically, ensuring LSP definitions stay in sync with the running version.
