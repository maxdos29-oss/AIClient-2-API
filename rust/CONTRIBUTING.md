# Contributing to AIClient-2-API Rust Version

Thank you for your interest in contributing to AIClient-2-API Rust! This document provides guidelines for contributing to this project.

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)
- Git

### Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/AIClient-2-API.git
   cd AIClient-2-API/rust
   ```

3. Create a new branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

4. Make your changes

5. Run tests:
   ```bash
   cargo test
   ```

6. Format your code:
   ```bash
   cargo fmt
   ```

7. Check for linting issues:
   ```bash
   cargo clippy
   ```

## Code Style

- Follow Rust's official style guidelines
- Use `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Add comments for complex logic
- Write unit tests for new functionality

## Pull Request Process

1. Update the README.md with details of changes if needed
2. Update the CHANGELOG.md (if exists)
3. Ensure all tests pass
4. Make sure your code follows the style guidelines
5. Create a pull request with a clear description of changes

## Testing

- Write unit tests for new functions
- Write integration tests for new endpoints
- Ensure existing tests still pass
- Aim for good test coverage

Example test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example() {
        // Test code here
    }
}
```

## Documentation

- Add documentation comments for public functions
- Use `///` for doc comments
- Include examples in doc comments when applicable
- Update README.md for significant changes

Example:

```rust
/// Converts data between different API formats
///
/// # Arguments
///
/// * `data` - The data to convert
/// * `from` - Source format
/// * `to` - Target format
///
/// # Returns
///
/// Converted data or error
///
/// # Examples
///
/// ```
/// let result = convert_data(data, Format::OpenAI, Format::Claude)?;
/// ```
pub fn convert_data(data: Value, from: Format, to: Format) -> Result<Value> {
    // Implementation
}
```

## Commit Messages

Follow conventional commits format:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `test:` Adding or modifying tests
- `chore:` Maintenance tasks

Example:
```
feat: add OpenAI adapter implementation
fix: resolve token refresh issue
docs: update README with new examples
```

## Questions?

Feel free to open an issue for any questions or concerns.

## License

By contributing, you agree that your contributions will be licensed under the GPL-3.0 License.

