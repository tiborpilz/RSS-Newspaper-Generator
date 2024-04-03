# RSS Newspaper Generator

Stores a list of RSS feeds which then can be transformed into a downloadable PDF using the readability library.

## Running your project

```bash
cargo leptos watch
```

## Compiling for Release
```bash
cargo leptos build --release
```

## Testing
```bash
cargo leptos end-to-end
```

```bash
cargo leptos end-to-end --release
```

Cargo-leptos uses Playwright as the end-to-end test tool.  
Tests are located in end2end/tests directory.