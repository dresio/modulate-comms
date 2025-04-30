![alt text](modulate.svg)

# Modulate - P2P Chat Application

Modulate is a peer-to-peer chat application with group chat functionality, built using Rust and WebRTC.

## Features

- Peer-to-peer communication using WebRTC
- No central server needed for messaging
- End-to-end encryption
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
cargo build
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

"WIP"

## Implementation Details

"WIP"
