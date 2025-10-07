# Changelog

All notable changes to the AIClient-2-API Rust version will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial Rust implementation framework
- Complete project architecture and module structure
- HTTP server with Axum framework
- Configuration management system
- API adapter interface and factory
- Format conversion framework
- Provider pool manager with load balancing
- Strategy pattern for different providers
- Provider modules (Gemini, OpenAI, Claude, Kiro, Qwen)
- Docker support with multi-stage build
- Docker Compose configuration
- Comprehensive documentation (README, ARCHITECTURE, CONTRIBUTING)
- Example configuration files
- Health check endpoint
- CORS support
- Multiple authentication methods
- Error handling system
- Logging with tracing framework

### Planned
- Complete implementation of all provider core logic
- OAuth 2.0 flow for Gemini, Kiro, and Qwen
- Full format conversion implementations
- System prompt management
- File logging support
- Periodic health checks
- Token refresh automation
- Unit and integration tests
- Performance benchmarks
- Monitoring and metrics
- WebSocket support
- GraphQL API support

## [1.0.0] - 2025-01-07

### Initial Release
- Project structure and framework
- Core module interfaces
- Documentation and deployment configurations

---

## Version History

### Framework (1.0.0)
This is the initial framework release. All core interfaces and architectures are defined and ready for implementation. The project structure follows Rust best practices and is designed for extensibility and maintainability.

### Future Releases
- **1.1.0**: OpenAI provider implementation
- **1.2.0**: Claude provider implementation
- **1.3.0**: Gemini provider implementation
- **1.4.0**: Kiro and Qwen provider implementations
- **1.5.0**: Complete format conversions
- **2.0.0**: Feature parity with Node.js version

---

**Note**: This is a living document. As features are implemented, they will be moved from "Planned" to the appropriate version section.

