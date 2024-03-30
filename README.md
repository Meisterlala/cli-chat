# Chat Client and Server
a very simple IRC clone using websockets and sqlite

You can read full german in death documentation [here](doc/german/main.pdf)


## What does it do?
The client connects to the server and sends messages to the server. The server then broadcasts the message to all connected clients. 

The client can choose a username and a "Group" or channel where the message will be send. This allows for multiple conversations to be happening at the same time on the same server. New clients can join the group and will receive all messages sent to that group.

Old messages are stored on the server in a sqlite database. 

## How to run it

Download the latest release from the releases page and run the server and client.

### Compile it yourself

1. Install rust from https://www.rust-lang.org/tools/install
2. Clone the repository
```bash
git clone https://github.com/Meisterlala/cli-chat.git
```
3. Compile the server and client
```bash
cd cli-chat
cargo build --release
```
4. Run the server
```bash
./target/release/chat_server
```
5. Run the client
```bash
./target/release/chat_client
```

## How to use it

### Server
Enter the port you want the server to listen on. 

### Client
Enter the IP and port of the server. Enter a username and a group. Then start chatting, when you press enter the message will be send. ESC or CTRL+C will close the client.

### Additional info
Its possible to see additional info by setting the environment variable `RUST_LOG=info` or `RUST_LOG=debug` before running the server or client. `RUST_LOG=off` will disable most output.


## Technical details
All messages are stored in a single sqlite database and table. The server uses tokio for async IO and the client uses ratatui for terminal IO. Envlogger is used for logging. Communication between server and client is done via tungstenite websockets.


# Screenshots
```
Chat Client, logged into "ws://127.0.0.1:9001" in the group "Hacker Chat" as "User 1"











 User 1: Hello, how are you?
 User 2: I'm doing fine
 User 2: How about you?
 User 1: Also good, check out this cool program
 User 1: we can type and messages appear in real time
 User 2: i think we live in the future, wow
┌Input───────────────────────────────────────────────────────────────────────────────────┐
│ very cool_                                                                             │
└────────────────────────────────────────────────────────────────────────────────────────┘
```
