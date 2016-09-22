# ArcadeRS

A simple old-school shooter made to explore the Rust programming language and ecosystem.
Made following the ArcadeRS tutorial at https://jadpole.github.io/arcaders/arcaders-1-0

Changed the version of the sdl2 dependency to 0.23.0, the latest version, and did the same for the sdl2_image dependency.
Having all sdl2 dependencies on the same version to prevent errors. 

When starting the game the main thread panics. The Sprite::load method in scr/phi/gfx.rs tries to load a texture but returns a None option, which the code attempts to unwrap in src/views/shared.rs when it tries to create the background.

