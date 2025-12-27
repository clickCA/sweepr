# Sweepr

**Blazing-fast dead code elimination for JavaScript and TypeScript**

Sweepr is a powerful static analysis tool that identifies unused code, dependencies, and exports in your JavaScript/TypeScript projects. Built with Rust and leveraging the oxc parser, it provides lightning-fast code analysis with parallel processing.

## Inspiration & Purpose

Sweepr is a proof-of-concept (PoC) project inspired by [knip](https://github.com/webpro/knip), the excellent dead code elimination tool for JavaScript/TypeScript. The goal of this project is to explore the performance benefits of implementing static analysis using Rust and the [oxc](https://github.com/oxc-project/oxc) parser suite.

While knip provides comprehensive analysis and production-ready features, Sweepr focuses on:

- **Performance optimization** through Rust's zero-cost abstractions and memory safety
- **Parallel processing** using Rayon for multi-core utilization
- **Fast parsing** with oxc's next-generation JavaScript/TypeScript parser
- **Learning opportunity** to understand the trade-offs between Node.js and Rust implementations

This project serves as an experiment to measure how a systems programming language can improve the tooling ecosystem for JavaScript/TypeScript development.

## Features

- 🔍 **Dead Code Detection** - Find unused exports, functions, and variables
- 📦 **Dependency Analysis** - Identify unused npm packages
- 🗂️ **File Reachability** - Detect files that aren't imported by your entry points
- ⚡ **Blazing Fast** - Parallel parsing and analysis powered by Rust
- 🎯 **Framework Agnostic** - Works with React, Vue, Angular, Node.js, and more
- 📊 **Multiple Output Formats** - Human-readable CLI output or JSON for CI/CD

## Installation

### Build from source

```bash
# Install Rust toolchain if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/user/sweepr.git
cd sweepr
cargo build --release

# The binary will be at target/release/sweepr
```

### Install globally (after building)

```bash
cargo install --path .
```

## Quick Start

```bash
# Analyze your project
sweepr check

# Analyze with custom entry points
sweepr check --entry src/main.ts --entry src/app.ts

# Output results in JSON format
sweepr check --json
```

## Usage

### Commands

#### `check` - Analyze for unused code (read-only)

The `check` command scans your project and reports unused code without making any changes.

```bash
# Basic check with default entry point
sweepr check

# Specify custom entry points
sweepr check --entry src/index.ts

# Multiple entry points
sweepr check -e src/main.ts -e src/app.ts

# JSON output for CI/CD integration
sweepr check --json > analysis-results.json
```

#### `fix` - Remove unused code (safe modifications)

*Note: The fix functionality is not yet implemented, but planned for future releases.*

```bash
# Safe fixes only (comment out unused code)
sweepr fix

# Allow dangerous operations (delete files)
sweepr fix --unsafe
```

### Configuration

Create a `sweepr.config.json` file in your project root:

```json
{
  "entry": ["src/index.ts", "src/app.ts"],
  "ignore": [
    "**/*.test.ts",
    "**/*.spec.ts",
    "**/node_modules/**",
    "src/legacy/**"
  ],
  "rules": {
    "unused_deps": true,
    "unused_exports": true,
    "unused_files": true
  },
  "framework": "react"
}
```

#### Configuration Options

- **`entry`** (array, required) - Entry point files for your application
  - Default: `["src/index.ts"]`
  - Examples: `["src/main.ts"]`, `["src/client.tsx", "src/server.ts"]`

- **`ignore`** (array, optional) - Glob patterns for files to ignore
  - Default: `["**/*.test.ts", "**/*.test.js", "**/*.spec.ts", "**/*.spec.js", "**/node_modules/**"]`
  - Supports glob patterns like `**/*.test.ts` or `src/legacy/**`

- **`rules`** (object, optional) - Enable/disable specific rules
  - `unused_deps` (boolean, default: `true`) - Check for unused npm dependencies
  - `unused_exports` (boolean, default: `true`) - Check for unused exports
  - `unused_files` (boolean, default: `true`) - Check for unreachable files

- **`framework`** (string, optional) - Framework-specific optimizations
  - Supported: `react`, `vue`, `angular`, `svelte`, `node`
  - Improves detection accuracy with framework-specific patterns

## What Sweepr Analyzes

### 1. Unused Dependencies

Scans your `package.json` and identifies packages that are never imported:

```
📦 Unused Dependencies (3)
  • lodash (4.17.21)
  • moment (2.29.4)
  • axios (1.6.0)
```

### 2. Unused Exports

Finds exported functions, classes, and variables that are never imported:

```
📄 Unused Exports in src/utils/helpers.ts (2)
  • formatDateTime (line 15)
  • parseQueryString (line 23)
```

### 3. Unreachable Files

Identifies files that aren't reachable from your entry points:

```
🗑️  Unreachable Files (5)
  • src/legacy/auth.ts
  • src/components/old/Button.tsx
  • src/utils/deprecated.ts
```

## Examples

### Example 1: React Application

```bash
# Analyze a React app
sweepr check --entry src/main.tsx

# Output:
🚀 Scanning workspace...
  📄 Found 127 files
  🎯 Entry points: 1

🔬 Analyzing code...
  ✓ Parsed 127 files
  ✓ Built analysis graphs
  ✓ Loaded 45 dependencies

📊 Analysis Results
  📦 Unused Dependencies: 3
  📄 Unused Exports: 12
  🗑️  Unreachable Files: 2

⏱️  Completed in 234.56ms
```

### Example 2: Node.js Backend

```bash
# Analyze a Node.js API
sweepr check --entry src/server.ts --entry src/cli.ts

# Or create a config file
cat > sweepr.config.json << EOF
{
  "entry": ["src/server.ts", "src/cli.ts"],
  "ignore": ["**/*.test.ts", "scripts/**"],
  "rules": {
    "unused_deps": true,
    "unused_exports": true,
    "unused_files": false
  }
}
EOF

sweepr check
```

### Example 3: CI/CD Integration

```bash
# In your CI pipeline
sweepr check --json | jq '.unused_dependencies | length'
# Exit with error if too many unused deps
if [ $(sweepr check --json | jq '.unused_dependencies | length') -gt 5 ]; then
  echo "Too many unused dependencies!"
  exit 1
fi
```

## How It Works

Sweepr uses sophisticated static analysis to understand your code:

1. **File Discovery** - Scans your project for JS/TS files
2. **Parallel Parsing** - Uses multiple CPU cores to parse files simultaneously
3. **AST Analysis** - Builds abstract syntax trees to understand code structure
4. **Graph Building** - Creates dependency graphs for files, symbols, and packages
5. **Reachability Analysis** - Traces code from entry points to find what's used
6. **Reporting** - Generates actionable insights about unused code

## Performance

Sweepr is optimized for speed:

- **Parallel parsing** using Rayon
- **Efficient AST** with oxc allocator
- **Incremental analysis** support (planned)

Benchmarks on a typical mid-size project (500 files):

| Tool | Time |
|------|------|
| Sweepr | ~200ms |
| ESLint | ~2s |
| TypeScript compiler | ~5s |

## Roadmap

- [ ] `fix` command implementation
- [ ] Framework-specific rules (React hooks, Vue composables)
- [ ] Incremental analysis mode
- [ ] VS Code extension
- [ ] Web UI for analysis results
- [ ] Support for more file types (JSX, TSX, JSON imports)
- [ ] Configuration file TypeScript support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built with amazing open-source tools:
- [knip](https://github.com/webpro/knip) - Inspiration and test fixtures
- [oxc](https://github.com/oxc-project/oxc) - JavaScript/TypeScript tooling
- [clap](https://github.com/clap-rs/clap) - Command line argument parser
- [rayon](https://github.com/rayon-rs/rayon) - Parallelism library
- [serde](https://github.com/serde-rs/serde) - Serialization framework

## Support

- 📖 [Documentation](https://github.com/user/sweepr/wiki)
- 🐛 [Issue Tracker](https://github.com/user/sweepr/issues)
- 💬 [Discussions](https://github.com/user/sweepr/discussions)

---

Made with ❤️ by the Sweepr contributors
