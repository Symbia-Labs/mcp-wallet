# Symbia Labs MCP Wallet

A secure desktop application for managing API integrations and exposing them via the Model Context Protocol (MCP). Convert any OpenAPI spec into MCP tools with encrypted credential storage.

## Features

- **Desktop Application**: Native Tauri app with React UI for managing integrations and credentials
- **OpenAPI to MCP Conversion**: Parse any OpenAPI 3.x spec and automatically generate MCP tools
- **Secure Credential Storage**: AES-256-GCM encryption with Argon2id key derivation, optional OS Keychain backing
- **Session-Based Authentication**: No passwords in config files - CLI uses secure session tokens
- **Dual MCP Transports**: stdio (Claude Desktop) and HTTP/SSE (network access)
- **Custom Integrations**: Add any API with an OpenAPI spec, not just predefined providers
- **OpenTelemetry Support**: Optional observability with configurable OTLP export

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri Desktop App                         │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              React UI (Vite + Tailwind)              │    │
│  │  - Unlock screen      - Integration management       │    │
│  │  - Credential vault   - Server status/control        │    │
│  │  - Settings           - OpenTelemetry config         │    │
│  └─────────────────────────────────────────────────────┘    │
│                          │ IPC                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                    Rust Core                         │    │
│  │  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  │    │
│  │  │ wallet-core │  │openapi-parser│  │ mcp-server │  │    │
│  │  │ - Crypto    │  │ - Parse 3.x  │  │ - stdio    │  │    │
│  │  │ - Storage   │  │ - Extract ops│  │ - HTTP/SSE │  │    │
│  │  │ - Sessions  │  │ - Namespace  │  │ - Execute  │  │    │
│  │  └─────────────┘  └──────────────┘  └────────────┘  │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
         │                                    │
         v                                    v
┌─────────────────┐                 ┌─────────────────┐
│   OS Keychain   │                 │  External APIs  │
│ (macOS/Win/Lin) │                 │ (Stripe, etc.)  │
└─────────────────┘                 └─────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 18+ (for UI development)
- Tauri CLI: `cargo install tauri-cli`

### Build

```bash
# Build the MCP server CLI
cargo build --release --package mcp-server

# Build the desktop app
cd crates/tauri-app && cargo tauri build
```

### Usage

1. **Launch the Desktop App**
   - Open `MCP Wallet.app` (macOS) or equivalent
   - Create a wallet with a secure password (first run) or unlock with your password

2. **Add Integrations**
   - Browse the catalog or click "Add Custom" to add any OpenAPI-compatible API
   - Enter the OpenAPI spec URL and optionally an API key

3. **Configure Claude Desktop**

   Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:
   ```json
   {
     "mcpServers": {
       "symbia-mcp-wallet": {
         "command": "/path/to/mcp-wallet-server",
         "args": ["--stdio"]
       }
     }
   }
   ```

   **No password needed!** The CLI automatically uses a secure session token created when you unlock the wallet in the desktop app. Sessions last 24 hours.

4. **Use MCP Tools in Claude**
   - Your integrations appear as tools (e.g., `stripe_customers_create`)
   - Claude can call these tools, and the wallet handles authentication

## Session-Based Authentication

MCP Wallet uses ephemeral session tokens instead of sharing your master password:

1. When you unlock the wallet in the desktop app, a 24-hour session is created
2. The session token is stored in `~/.mcp-wallet/session.json`
3. The MCP server CLI uses this token to decrypt credentials without needing the password
4. When you lock the wallet, the session is cleared and CLI access is revoked

This means:
- No passwords in config files or environment variables
- MCP access is automatically revoked when you lock the wallet
- You can close the desktop app while keeping MCP tools working (session persists)

## Storage Locations

| Platform | Data Directory |
|----------|----------------|
| macOS    | `~/Library/Application Support/com.symbia-labs.mcp-wallet/` |
| Linux    | `~/.local/share/mcp-wallet/` |
| Windows  | `%APPDATA%\symbia-labs\mcp-wallet\` |

Contents:
- `wallet.json` - Encrypted integrations and credentials
- `salt` - Key derivation salt
- `verify` - Password verification data
- `session.json` - Current session token (if unlocked)
- `settings.json` - Non-sensitive configuration (OTEL, auto-lock, etc.)

## Crates

| Crate | Description |
|-------|-------------|
| `wallet-core` | Encryption, storage, sessions, settings, integration registry, credential management |
| `openapi-parser` | OpenAPI 3.x parsing and operation extraction |
| `mcp-server` | MCP protocol implementation (stdio + HTTP/SSE transports) |
| `tauri-app` | Desktop application with React UI |

## Security Model

### Master Key Derivation

```
Password → Argon2id(password, salt, t=3, m=64MB, p=4) → 256-bit key
```

### Session Token Security

```
Session Token (random 32 bytes) → hex-encoded (64 chars)
Master Key → AES-256-GCM(token_as_key) → Encrypted in session.json
```

### Credential Encryption

```
Plaintext API Key → AES-256-GCM(master_key, random_iv) → iv:tag:ciphertext
```

### Memory Safety

- All secrets wrapped in `Zeroize` types
- Cleared from memory on drop
- No sensitive data in logs
- Session cleared on wallet lock

## MCP Tool Naming Convention

Tools are named `{integration}_{normalized_operation_id}`:

- `stripe_customers_create` → POST /v1/customers
- `openai_chat_completions_create` → POST /v1/chat/completions
- `github_repos_list` → GET /repos

## Configuration

### OpenTelemetry

Configure observability in Settings → Observability:

- **OTLP Endpoint**: gRPC or HTTP endpoint (e.g., `http://localhost:4317`)
- **Service Name**: Defaults to `symbia-mcp-wallet`
- **Auth Header**: For cloud providers (Honeycomb, Grafana Cloud, etc.)
- **Export Options**: Toggle traces and/or metrics

### Auto-Lock

Configure wallet auto-lock timeout in Settings → Security.

## Development

### Run Tests

```bash
cargo test
```

### Build UI in Development

```bash
cd crates/tauri-app/ui
npm install
npm run dev
```

### Run Desktop App in Development

```bash
cd crates/tauri-app
cargo tauri dev
```

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.

---

Built by [Symbia Labs](https://www.symbia-labs.com)
