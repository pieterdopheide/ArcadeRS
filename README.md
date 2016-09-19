ArcadeRS

A simple old-school shooter made to explore the Rust programming language and ecosystem.
Made following the ArcadeRS tutorial at https://jadpole.github.io/arcaders/arcaders-1-0

Changed the version of the sdl2 dependency from 0.23.0, the latest version, to 0.13. This was done because of a compile error stating that there was no unwrap method for sdl2::rect:Rect.

