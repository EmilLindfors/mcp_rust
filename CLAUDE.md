# MCP Rust Project Guidelines

## Build Commands
- Build project: `cargo build`
- Run project: `cargo run`
- Release build: `cargo build --release`

## Test Commands
- Run all tests: `cargo test`
- Run specific test: `cargo test test_name`
- Run tests in a module: `cargo test module_name`

## Lint/Format Commands
- Format code: `cargo fmt`
- Run linter: `cargo clippy`
- Fix lint issues: `cargo clippy --fix`

## Code Style Guidelines
- Follow Rust idioms and conventions in the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use snake_case for variables and functions
- Use PascalCase for types and structs
- Handle errors using Result<T, E> with proper error propagation via `?` operator
- Prefer Result over panic/unwrap in production code
- Organize imports alphabetically and group standard library, external crates, and internal modules
- Document public APIs with rustdoc comments (///)
- Use internal comments (//) for complex logic explanations
- Keep functions small and focused on a single responsibility