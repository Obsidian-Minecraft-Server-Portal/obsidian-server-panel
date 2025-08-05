# Obsidian Minecraft Server Panel

A modern web-based management panel for Minecraft servers with Forge support, featuring a responsive React frontend and a Rust backend.

## Features

- **Minecraft Server Management**: Control your Minecraft servers directly from the web interface
- **Forge Version Management**: Fetch and display available Forge versions from the official Minecraft Forge website
- **Authentication System**: Secure access to server management features
- **Responsive Web Interface**: Built with React and TypeScript for a modern user experience
- **UPnP Port Forwarding**: Automatically open ports for your Minecraft server (development mode)
- **Database Integration**: Persistent storage for server configurations and user data

## Prerequisites

- Rust 1.87.0 or later
- Node.js 20.x or later (for frontend development)
- pnpm package manager
- Cargo (Rust's package manager)

## Building the Project

### Backend (Rust)

1. Install dependencies:

```shell script
cargo build
```

2. For development with hot reloading:

```shell script
cargo run
```

### Frontend (React/TypeScript)

1. Install Node.js dependencies:

```shell script
pnpm install
```

2. Build the frontend for production:

```shell script
pnpm build
```

3. Run in development mode:

```shell script
pnpm dev
```

## Running the Application

### Development Mode

To run the application in development mode with hot reloading:

```shell script
cargo run
```

This will start both the Rust backend server and the Vite frontend development server.

### Production Mode

To run in production mode:

```shell script
cargo build --release
./target/release/obsidian-server-panel
```

## Command Line Arguments

| Argument             | Description                                   | Default |
|----------------------|-----------------------------------------------|---------|
| `--port`             | Port to run the web server on                 | `8080`  |
| `--forward-webpanel` | Enable UPnP port forwarding for the web panel | `false` |

Example usage:

```shell script
cargo run -- --port 3000 --forward-webpanel
```

## Key Components

### Backend (Rust)

- **Actix Web**: HTTP server framework
- **SQLx**: Database connectivity and queries
- **Serde**: Serialization/deserialization of data
- **Clap**: Command-line argument parsing
- **Tokio**: Asynchronous runtime

### Frontend (React/TypeScript)

- **React 18.2.0**: Component-based UI library
- **Vite 6.0.11**: Fast build tool and development server
- **Tailwind CSS**: Utility-first CSS framework
- **React Router DOM**: Client-side routing
- **Framer Motion**: Animation library

## Directory Structure

```
.
├── src/
│   ├── lib.rs          # Main entry point
│   ├── app_db/         # Database integration
│   ├── authentication/ # Auth middleware and endpoints
│   ├── command_line_args/ # Command line argument parsing
│   ├── ddos_middleware/ # DDoS protection
│   ├── forge_endpoint/  # Forge version API endpoint
│   ├── host_info/       # System information endpoints
│   ├── java/           # Java-related functionality
│   └── server/         # Server management
├── resources/          # Static assets (logo, etc.)
├── frontend/           # React/TypeScript frontend code
└── Cargo.toml          # Rust dependencies and configuration
```

## Database Setup

The application uses SQLite for local development and can be configured to use PostgreSQL in production. The database schema is initialized automatically during startup.

## Security Notes

- In development mode, the application automatically sets up UPnP port forwarding for convenience
- Authentication is required for all server management endpoints
- All sensitive information should be stored securely (not shown in this README)

## License

This project is licensed under the obsidian-private-license - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please follow standard Rust and React development practices.