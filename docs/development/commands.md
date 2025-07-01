# Development Commands Reference

This document provides a comprehensive guide to all available commands for developing, building, testing, and maintaining the Claudia project.

## Command Categories

### Development Commands

#### Frontend Development Server
```bash
bun run dev
```
Starts the Vite frontend development server only. This is useful when you're working exclusively on frontend features and don't need the full Tauri application running.

- **Use case**: Frontend component development, styling, React debugging
- **Hot reload**: Yes
- **Port**: Usually 5173 (configurable in vite.config.ts)

#### Full Application Development
```bash
bun run tauri dev
```
Starts the complete Tauri application in development mode, including both frontend and Rust backend.

- **Use case**: Full application development, testing Tauri commands, debugging frontend-backend communication
- **Hot reload**: Frontend only (Rust changes require restart)
- **Features**: All Tauri APIs available, window management, system integration

### Build Commands

#### Frontend Build
```bash
bun run build
```
Performs TypeScript type checking and builds the frontend using Vite. This creates optimized production assets without building the full desktop application.

- **Output**: `dist/` directory with static assets
- **Type checking**: Yes
- **Optimization**: Production minification and bundling

#### Application Build
```bash
bun run tauri build
```
Builds the complete Tauri desktop application for distribution.

- **Output**: Platform-specific installers/binaries in `src-tauri/target/release/`
- **Platforms**: macOS (.app, .dmg), Windows (.exe, .msi), Linux (.AppImage, .deb)
- **Requirements**: Platform-specific build tools (Xcode on macOS, etc.)

### Testing Commands

#### Frontend Tests

##### Watch Mode
```bash
bun run test
```
Runs frontend tests in watch mode using Vitest. Tests automatically re-run when files change.

##### UI Mode
```bash
bun run test:ui
```
Opens Vitest's interactive UI for exploring and debugging tests visually.

##### Single Run
```bash
bun run test:run
```
Executes all tests once without watch mode. Useful for CI/CD pipelines.

##### Coverage Report
```bash
bun run test:coverage
```
Generates a test coverage report showing which parts of your code are tested.

- **Output**: Coverage reports in `coverage/` directory
- **Formats**: HTML, JSON, text summaries

#### Backend Tests
```bash
cd src-tauri && cargo test
```
Runs the Rust backend test suite.

- **Includes**: Unit tests, integration tests, doc tests
- **Parallel**: Tests run in parallel by default
- **Filtering**: Use `cargo test <pattern>` to run specific tests

### Code Quality Commands

#### Rust Formatting
```bash
cd src-tauri && cargo fmt
```
Formats all Rust code according to the project's rustfmt configuration.

- **Config**: `.rustfmt.toml` in src-tauri directory
- **Check mode**: Add `--check` to verify formatting without changes

### Preview Commands

#### Production Preview
```bash
bun run preview
```
Serves the production build locally for testing before deployment.

- **Prerequisite**: Run `bun run build` first
- **Use case**: Testing production optimizations, verifying build output

## Command Workflow Examples

### Daily Development Workflow
```bash
# 1. Start development server
bun run tauri dev

# 2. Run tests in another terminal
bun run test

# 3. Format code before committing
cd src-tauri && cargo fmt
```

### Pre-Release Checklist
```bash
# 1. Run all tests
bun run test:run
cd src-tauri && cargo test

# 2. Check coverage
bun run test:coverage

# 3. Build and preview
bun run build
bun run preview

# 4. Final production build
bun run tauri build
```

### Debugging Workflow
```bash
# Frontend only (faster startup)
bun run dev

# With Rust debugging
RUST_LOG=debug bun run tauri dev
```

## Environment Variables

Some commands support environment variables for additional configuration:

- `RUST_LOG`: Set Rust logging level (error, warn, info, debug, trace)
- `VITE_PORT`: Override default Vite dev server port
- `TAURI_DEBUG`: Enable Tauri debug mode

## Troubleshooting

### Common Issues

1. **Build fails with "cargo not found"**
   - Install Rust: https://rustup.rs/

2. **Tauri build fails on macOS**
   - Ensure Xcode Command Line Tools are installed: `xcode-select --install`

3. **Frontend tests fail with module errors**
   - Clear node_modules and reinstall: `rm -rf node_modules && bun install`

4. **Port already in use**
   - Kill the process or change port in vite.config.ts

## Additional Tools

While not part of the main commands, these tools are useful for development:

```bash
# Analyze bundle size
bun run build -- --analyze

# Type check without building
bun tsc --noEmit

# Lint Rust code
cd src-tauri && cargo clippy

# Update dependencies
bun update
cd src-tauri && cargo update
```