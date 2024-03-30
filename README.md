# Chat Client and Server
a very simple IRC clone


## What does it do?
The client connects to the server and sends messages to the server. The server then broadcasts the message to all connected clients. 

The client can choose a username and a "Group" or channel where the message will be send. This allows for multiple conversations to be happening at the same time on the same server. New clients can join the group and will receive all messages sent to that group.

Old messages are stored on the server in a sqlite database. 