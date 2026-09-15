# rust-localserver

A lightweight, async HTTP/1.1 local server written from scratch in Rust. It supports static file serving, directory listing, virtual hosting, CGI script execution, and custom error pages — all configured via a simple INI file.

## Features

- **Static file serving** — serves files from a configurable root directory
- **Directory listing** — optionally renders browsable directory indexes
- **Virtual hosting** — multiple named servers on the same address/port, routed by `Host` header
- **CGI execution** — runs executable scripts under a configurable `cgi_root` path
- **Custom error pages** — maps HTTP error status codes to HTML files in a configurable directory
- **Client body size limiting** — rejects request bodies that exceed a configurable threshold
- **Custom async runtime** — built-in cooperative task pool and async I/O primitives (no Tokio dependency)
- **INI-based configuration** — one config file drives the entire server

## Project Structure

```
rust-localserver/
├── src/
│   ├── main.rs            # Entry point: parses config, sets up bindings, runs the server pool
│   ├── lib.rs             # Public module re-exports
│   ├── handler.rs         # Top-level request dispatcher (file server vs CGI)
│   ├── http/              # HTTP types, request/response, server, parser, cookies
│   ├── file_server/       # Static file & directory listing logic
│   ├── cgi_executor/      # CGI script execution and environment setup
│   ├── parser/            # INI config file parser and ServerConfig structs
│   └── future/            # Custom async runtime: tasks, pool, async TCP/read/write
├── tests/
│   └── unit_tests.rs      # Integration & unit tests (request parsing, config, routing)
├── examples/
│   └── fileserver/        # Example server root
│       ├── index.html
│       ├── cgi/
│       │   └── hello_world.sh
│       └── pages/errors/  # Custom error HTML pages
└── config.ini             # Server configuration file
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024, stable toolchain)

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run --release
```

The server reads `config.ini` from the current working directory on startup.

### Run Tests

```bash
cargo test
```

## Configuration

The server is configured through `config.ini` using INI syntax. Each server block is declared as `[server.<name>]`.

### Example `config.ini`

```ini
[server.fileserver]
address=127.0.0.1
port=80
host=fileserver.srm
root=examples/fileserver
list_directory=true
index=index.html
error_pages_dir=pages/errors
client_body_size_limit=10GB
cgi_root=/cgi
```

### Configuration Options

| Option | Description | Example |
|---|---|---|
| `address` | IP address to bind | `127.0.0.1` |
| `port` | TCP port to listen on | `8080` |
| `host` | Hostname(s) for virtual hosting (separate multiple with `/`) | `example.com/www.example.com` |
| `root` | Path to the document root directory | `examples/fileserver` |
| `index` | Default index file to serve for directory requests | `index.html` |
| `list_directory` | Enable directory listing when no index file is found | `true` / `false` |
| `error_pages_dir` | Path (relative to `root`) for custom error HTML pages | `pages/errors` |
| `client_body_size_limit` | Maximum allowed request body size | `10MB`, `1GB` |
| `cgi_root` | URL path prefix that triggers CGI execution | `/cgi` |

### Virtual Hosting

Multiple `[server.<name>]` blocks can share the same `address`/`port`. Requests are routed to the correct server by matching the HTTP `Host` header. Duplicate hostnames on the same binding are detected and rejected at startup.

```ini
[server.app]
address=0.0.0.0
port=8080
host=app.local
root=my_app/public
...

[server.api]
address=0.0.0.0
port=8080
host=api.local
root=my_api/public
...
```

## CGI Execution

Any request whose resolved path falls under `<root>/<cgi_root>` is executed as a CGI script instead of being served as a static file.

The following standard CGI environment variables are set for the script:

| Variable | Value |
|---|---|
| `REQUEST_METHOD` | HTTP method (`GET`, `POST`, …) |
| `SCRIPT_NAME` | Request path |
| `QUERY_STRING` | URL query string |
| `CONTENT_TYPE` | Value of the `Content-Type` request header |
| `CONTENT_LENGTH` | Value of the `Content-Length` request header |
| `HTTP_*` | All other request headers, uppercased and prefixed with `HTTP_` |

The script's stdout is forwarded directly as the HTTP response. The script's working directory is set to its own directory.

### CGI Example

```bash
#!/bin/sh
echo "Content-Type: text/plain"
echo ""
echo "Hello, World!"
```

> **Note:** CGI scripts must be executable (`chmod +x`).

## Architecture

### Async Runtime

The server uses a custom cooperative async runtime (`src/future/`) built on top of `std::future`. It provides:

- **`Task`** — wraps a `Future` into a pinned, pollable unit of work
- **`Pool`** — a cooperative task scheduler that drives tasks to completion
- **`AsyncTcpStream`** / **`AsyncTcpListener`** — non-blocking wrappers over std TCP types
- **`ReadFuture`** / **`WriteFuture`** — async I/O primitives

### Request Lifecycle

```
TCP Connection
      │
      ▼
  HTTP Parser  ──► RequestParser reads start line + headers
      │
      ▼
  server_handler
      │
      ├── Match Host header → find ServerConfig
      │
      ├── Path starts with cgi_root?
      │       ├── Yes → CGIExecutor::exec (async stdin/stdout bridge)
      │       └── No  → FileServer::serve (static file or directory listing)
      │
      └── Write HTTP response (status + headers + body)
```

## Dependencies

| Crate | Purpose |
|---|---|
| [`bytesize`](https://crates.io/crates/bytesize) | Human-readable byte size parsing for `client_body_size_limit` |
| [`libc`](https://crates.io/crates/libc) | `fcntl` syscall for setting file descriptors to non-blocking mode |

> Dev dependencies (`bcrypt`, `form_urlencoded`, `getrandom`, `hex`) are used only in tests and examples.

## License

This project is open source. See your repository's license file for details.
