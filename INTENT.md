# MCP Wallet - Intent & Design Philosophy

## Purpose

MCP Wallet exists to solve a fundamental problem: **AI assistants need secure access to APIs, but sharing API keys is risky.**

When you give Claude access to your Stripe API key, you're trusting that:
1. The key won't be logged or leaked
2. The key will be encrypted at rest
3. Access can be revoked when needed
4. The key won't be exposed in config files

MCP Wallet is a secure gateway that:
- Stores your API credentials with hardware-grade encryption
- Exposes them to AI assistants via MCP (Model Context Protocol) without revealing the actual keys
- Converts any OpenAPI spec into MCP tools automatically
- Provides a desktop app for managing everything visually

## Design Principles

### 1. Security First, Convenience Second

Every design decision prioritizes security:

- **No passwords in config files**: Session tokens are used instead, created on unlock and cleared on lock
- **Hardware-backed encryption when available**: macOS Keychain, Windows DPAPI, Linux Secret Service
- **Zero-knowledge design**: The wallet never sends credentials anywhere except to the target API
- **Zeroize on drop**: All secrets are cleared from memory when no longer needed

### 2. OpenAPI as the Universal Language

Rather than building custom integrations for each API, we use OpenAPI specs as the source of truth:

- Any API with an OpenAPI 3.x spec can be added
- Operations are automatically converted to MCP tools
- Parameters, authentication, and request bodies are handled automatically
- No code changes needed to support new APIs

### 3. MCP as the Access Layer

The Model Context Protocol provides a standardized way for AI to interact with tools:

- **stdio transport**: Direct process communication for Claude Desktop
- **HTTP/SSE transport**: Network access for other clients
- Tools are namespaced by integration (e.g., `stripe_customers_create`)
- The wallet handles all authentication transparently

### 4. Desktop-First Experience

While the MCP server runs headless, the wallet is managed through a desktop app:

- Visual management of integrations and credentials
- Clear status indicators for what's working
- No command-line setup required for end users
- Settings and configuration in one place

## Architecture Decisions

### Why Tauri + Rust?

- **Security**: Rust's memory safety prevents common vulnerability classes
- **Performance**: Native code with minimal overhead
- **Cross-platform**: Single codebase for macOS, Windows, Linux
- **Small binaries**: Tauri apps are much smaller than Electron

### Why Session Tokens?

The original design required passing a password to the MCP server. This created problems:

1. Password in environment variables (visible in process list)
2. Password in config files (readable by other processes)
3. No way to revoke access without changing the password

Session tokens solve all three:

1. Token is stored in a protected file, not env vars
2. Token is ephemeral - created on unlock, cleared on lock
3. Locking the wallet immediately revokes CLI access

### Why Settings Separate from Encrypted Storage?

Settings like OTEL endpoints and auto-lock timeouts don't need encryption:

- They contain no secrets
- They should be accessible when the wallet is locked (for UI display)
- They don't need the complexity of encrypted storage

So settings live in a plain JSON file alongside the encrypted wallet.

### Why Property Name Sanitization?

Claude's MCP implementation requires property names to match `^[a-zA-Z0-9_.-]{1,64}$`. OpenAPI specs often have property names with brackets, special characters, or excessive length. The generator sanitizes these automatically to ensure compatibility.

## Key Flows

### Unlock Flow

```
User enters password
    → Argon2id derives master key from password + salt
    → Master key decrypts verification data
    → If verification passes, master key is held in memory
    → Session token created (random 32 bytes)
    → Master key encrypted with token, stored in session.json
    → Integrations and credentials loaded from encrypted storage
```

### MCP Tool Call Flow

```
Claude calls stripe_customers_create with arguments
    → MCP server receives tool call
    → Loads session from disk, derives master key from token
    → Looks up integration and credential for "stripe"
    → Decrypts API key
    → Constructs HTTP request from OpenAPI operation + arguments
    → Adds authentication header with decrypted API key
    → Executes request against Stripe API
    → Returns response to Claude
    → API key zeroized from memory
```

### Lock Flow

```
User clicks Lock (or auto-lock timeout)
    → Master key cleared from wallet memory
    → Session file deleted
    → State set to Locked
    → Any running MCP server calls will fail with "session expired"
```

## Future Directions

### OAuth2 Support

Currently only API key auth is fully implemented. OAuth2 flows would allow:
- Token refresh handling
- Scope management
- User consent flows

### Multi-User Support

The current design is single-user. Multi-user would need:
- Per-user credential isolation
- Shared vs. private integrations
- Access control lists

### Audit Logging

For enterprise use:
- Log all tool calls (without credentials)
- Track which AI assistant accessed which API
- Compliance reporting

### Remote Management

For teams:
- Central credential management
- Distribute integrations to team members
- Revoke access centrally

## Non-Goals

### Not a General Password Manager

MCP Wallet specifically manages API credentials for AI access. It doesn't:
- Store passwords for websites
- Generate passwords
- Sync across devices
- Share with family members

Use 1Password, Bitwarden, etc. for general password management.

### Not an API Gateway

MCP Wallet doesn't:
- Rate limit API calls
- Cache responses
- Transform data
- Provide API analytics

It's a secure credential store with MCP protocol support, not a full API gateway.

### Not a Development Tool

MCP Wallet is for end users who want AI assistants to access their APIs. It doesn't:
- Generate API clients
- Mock API responses
- Test API endpoints
- Document APIs

Use Postman, Insomnia, or similar tools for API development.

## Summary

MCP Wallet bridges the gap between powerful AI assistants and the APIs they need to access, doing so with security as the primary concern. It treats API credentials as sensitive data deserving hardware-grade protection, while making it easy for non-technical users to set up and manage their integrations through a friendly desktop interface.

The combination of OpenAPI parsing, MCP protocol support, and session-based authentication creates a secure, flexible system that can work with virtually any API without requiring custom code or exposing credentials in config files.

---

*This document describes the intent behind MCP Wallet's design. For usage instructions, see [README.md](README.md).*
