# Testing Sweepr

This directory contains test fixtures and integration tests for Sweepr.

## Test Fixtures

The `tests/fixtures/` directory contains a comprehensive collection of test scenarios adapted from the knip project. These fixtures cover various JavaScript/TypeScript project configurations and edge cases.

### Using Test Fixtures

#### Quick Test with a Fixture

```bash
# Build the project
cargo build

# Test with the dependencies fixture
cd tests/fixtures/dependencies
../../../target/debug/sweepr check --entry entry.ts

# Or use the test runner script
./tests/test-runner.sh
```

#### Individual Fixtures

Each fixture directory is a self-contained test case:

- **`dependencies/`** - Tests dependency detection (unused vs used packages)
- **`exports/`** - Tests export analysis and re-exports
- **`imports/`** - Tests various import patterns
- **`re-exports/`** - Tests re-export scenarios
- **`workspaces/`** - Tests monorepo/workspace setups
- And many more...

### Fixture Structure

A typical fixture contains:

```
fixture-name/
├── package.json           # npm dependencies
├── tsconfig.json          # TypeScript config (optional)
├── knip.json             # Expected configuration (optional)
├── entry.ts              # Entry point
├── used-module.ts        # Module that should be detected as used
└── unused-module.ts      # Module that should be detected as unused
```

## Running Tests

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
# Run all tests
cargo test --test integration_test

# Run specific test
cargo test test_dependencies_fixture
```

### Manual Testing with Fixtures

```bash
# Test specific fixture
cd tests/fixtures/dependencies
cargo run -- check --entry entry.ts

# Test with custom entry points
cargo run -- check --entry src/main.ts --entry src/app.ts

# JSON output for CI/CD
cargo run -- check --json > results.json
```

## Writing New Fixtures

When creating a new test fixture:

1. **Create the directory structure:**
   ```bash
   mkdir tests/fixtures/my-test-case
   cd tests/fixtures/my-test-case
   ```

2. **Add necessary files:**
   - `package.json` - Define dependencies
   - Entry point file(s)
   - Source files with specific patterns to test
   - Configuration files as needed

3. **Document expected behavior:**
   ```bash
   cat > README.md << 'EOF'
   # My Test Case

   ## Purpose
   Tests that Sweepr correctly identifies...

   ## Entry Points
   - entry.ts

   ## Expected Results
   - Unused dependencies: []
   - Unused exports: 2
   - Unused files: 1
   EOF
   ```

4. **Add integration test:**
   ```rust
   #[test]
   fn test_my_test_case() {
       // Test implementation
   }
   ```

## Common Test Scenarios

### 1. Unused Dependencies

**Fixture:** `tests/fixtures/dependencies`

Tests that Sweepr can:
- Detect packages in `package.json` that are never imported
- Distinguish between used and unused dependencies
- Handle peer dependencies and dev dependencies

### 2. Unused Exports

**Fixture:** `tests/fixtures/exports`

Tests that Sweepr can:
- Find exported functions that are never imported
- Handle named exports, default exports, and re-exports
- Track exports across files

### 3. File Reachability

**Fixture:** `tests/fixtures/entry-files`

Tests that Sweepr can:
- Determine which files are reachable from entry points
- Identify files that cannot be reached from any entry
- Handle circular dependencies

### 4. Complex Imports

**Fixture:** `tests/fixtures/re-exports`

Tests that Sweepr can:
- Follow re-export chains
- Handle namespace imports
- Resolve indirect imports

## Debugging Failed Tests

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run -- check --entry entry.ts
```

### Check Parsed AST

```bash
# Add debug prints in src/parser/mod.rs
# Rebuild and run
cargo build
./target/debug/sweepr check --entry entry.ts
```

### Inspect Graph Structures

Add temporary debug output to see graph state:

```rust
// In src/rules/mod.rs
println!("Reachable files: {:?}", reachable);
println!("All files: {:?}", file_graph.files);
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Test Sweepr

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test
      - name: Test fixture
        run: |
          cd tests/fixtures/dependencies
          cargo run -- check --entry entry.ts
```

## Known Issues

### Current Limitations

1. **Relative Import Resolution** - Relative imports may not correctly resolve to absolute paths in the file graph
2. **Type-Only Imports** - Type-only imports (`import type { X }`) are tracked as regular imports
3. **Dynamic Imports** - Dynamic `import()` expressions may not be fully analyzed
4. **Framework-Specific Patterns** - React, Vue, etc. specific patterns may not be recognized

### Test Cases Showing These Issues

See individual fixture directories for specific examples of edge cases.

## Contributing Test Fixes

When fixing a failing test:

1. Identify the root cause in the source code
2. Add unit tests for the specific functionality
3. Update the integration test to verify the fix
4. Document the change in the test README
5. Ensure all other tests still pass

## Performance Benchmarks

Run performance tests on large fixtures:

```bash
# Time the analysis
time cargo run -- check --entry entry.ts

# Measure memory usage
/usr/bin/time -v cargo run -- check --entry entry.ts
```

## Resources

- [oxc AST Documentation](https://github.com/oxc-project/oxc)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [knip Test Fixtures](https://github.com/webpro/knip/tree/main/test/fixtures) - Original source of many fixtures
