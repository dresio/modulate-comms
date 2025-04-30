```
cargo build
cargo run -- offer
cargo run -- answer
```

# Modulate - P2P Chat Application

Modulate is a peer-to-peer chat application with group chat functionality, built using Rust and WebRTC.

## Features

- Peer-to-peer communication using WebRTC
- No central server needed for messaging
- End-to-end encryption
- Multiple connection modes (direct or via signaling server)
- Low latency messaging

## Setup

### Prerequisites

- Rust toolchain (1.70.0 or newer)
- Cargo package manager

### Installation

1. Clone the repository:

```bash
git clone https://github.com/dresio/modulate.git
cd modulate
```

2. Build the application:

```bash
cargo build --release
```

3. Run the application:

```bash
./target/release/modulate --help
```

## Usage

### Direct Connection Mode

#### Offerer (initiates the connection):

```bash
./target/release/modulate offer
```

#### Answerer (waits for an offer):

```bash
./target/release/modulate answer
```

### Group Chat Mode (Experimental)

```bash
./target/release/modulate group --max-peers 5
```

## Chat Commands

Once in a chat session, the following commands are available:

- `/exit` or `/quit` - Exit the chat
- `/help` - Show help message
- `/status` - Show connection status
- `/clear` - Clear the screen
- `/history` - Show message history

## Project Structure

- `main.rs` - Main application logic
- `signaling.rs` - Signaling server implementation
- `signaling-server.html` - Static HTML/JS signaling server

## Implementation Details

### WebRTC Connection

The application uses WebRTC for peer-to-peer connections with the following components:

- ICE (Interactive Connectivity Establishment) for NAT traversal
- STUN servers for public IP discovery
- TURN servers (configurable) for fallback when direct connection is not possible
- Data channels for reliable message transport

### Signaling Process

1. The offerer creates a WebRTC offer and ICE candidates
2. These are transmitted to the answerer via the signaling server or manual copy-paste
3. The answerer creates an answer and ICE candidates and sends them back
4. Once both peers have exchanged information, the WebRTC connection is established
5. Further communication happens directly between peers without intermediaries

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
