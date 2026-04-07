# Obsidian-UPNP

`obsidian-upnp` is a simple, lightweight wrapper for UPnP (Universal Plug and Play), designed for the [Obsidian Minecraft Server Portal](https://github.com/Obsidian-Minecraft-Server-Portal). This library simplifies managing port forwarding for Minecraft servers by providing utilities to automate adding, maintaining, and clearing UPnP port mappings.

---

## Features

- Add, renew, and delete UPnP port mappings programmatically.
- Automatically refresh active port mappings to ensure they stay active.
- Thread-safe singleton for managing multiple port mappings.
- Macro-based API for a simple and intuitive developer experience.

---

## Requirements

- Rust **1.82.0** or higher.
- This library can only be included as a git dependency. Ensure your `Cargo.toml` includes the following:

```toml
[dependencies]
obsidian-upnp = { git = "https://github.com/Obsidian-Minecraft-Server-Portal/upnp.git" }
```

---

## Usage

Add this library as a dependency in your `Cargo.toml` and use the provided API or macros to manage ports.

### Basic Example

```rust
use obsidian_upnp::{open_port, close_port, close_all_ports};

fn main() {
    // Open a port with the specified description
    open_port!(25565, "Minecraft Server");

    // Close the specified port
    close_port!(25565);

    // Close all opened ports
    close_all_ports!();
}
```

### Key Modules and Macros:

- **Macros**:
    - `open_port!(port, description)`: Opens and maintains the port.
    - `close_port!(port)`: Removes a specific port mapping.
    - `close_all_ports!()`: Clears all currently active port mappings.

- **UPnP Singleton**:
    - Thread-safe singleton for managing open ports: `UPNP_SINGLETON`.

---

## API Details

### Adding a Port

The `open_port!` macro creates and refreshes a UPnP port mapping at a specific interval (default: every 5 minutes). This happens in the background, ensuring your port remains open without manual intervention.

```rust
open_port!(port_number, "Port description");
```

- `port_number` - The port to open.
- `"Port description"` - A comment for the UPnP mapping (e.g., `"Minecraft Server"`).

Example:

```rust
open_port!(25565, "Minecraft Server");
```

### Removing a Port

Use the `close_port!` macro to delete a UPnP mapping for a specific port.

```rust
close_port!(port_number);
```

Example:

```rust
close_port!(25565);
```

### Clearing All Ports

To remove all active UPnP port mappings:

```rust
close_all_ports!();
```

---

## Under the Hood

The library leverages the [`easy-upnp`](https://crates.io/crates/easy-upnp) crate for interacting with UPnP and the [`obsidian-scheduler`](https://github.com/Obsidian-Minecraft-Server-Portal/scheduler) library to handle scheduled tasks (e.g., refreshing port mappings).

Key components:

- **Singleton Management**: The `UPNP_SINGLETON` ensures thread-safe operations using `lazy_static` + `Mutex`.
- **Automated Refresh**: Port mappings are re-added every 5 minutes to make sure they remain active.
- **Port Cleanup**: Ports are automatically removed by the `clear_ports` or `remove_port` methods.

---

## Installation

To add this library to your project, include it as a **git dependency** in your `Cargo.toml`:

```toml
[dependencies]
obsidian-upnp = { git = "https://github.com/Obsidian-Minecraft-Server-Portal/upnp.git" }
```

Run `cargo build` to fetch and compile the library.

---

## License

This library is licensed under the [MIT License](./LICENSE).

---

## Contribution

Feel free to contribute to this project via pull requests! Please follow the contribution guidelines specified in the repository.

---

## Links

- [GitHub Repository](https://github.com/Obsidian-Minecraft-Server-Portal/upnp)
- [Documentation](#) (coming soon)
- [Obsidian Scheduler](https://github.com/Obsidian-Minecraft-Server-Portal/scheduler)
- [Easy-UPnP](https://crates.io/crates/easy-upnp)

---