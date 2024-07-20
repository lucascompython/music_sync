# Music Sync

It's a simple tool that I made to sync my music library between my devices.

## How does it work?

There is a server that hosts the music library and clients that sync the music library with the server.  
The clients may also update the music library on the server.

## Security

Because the clients can update the music library on the server and thus write an important amount of data, there is a token authentication system.  
Keep in mind that the data is not encrypted, because I don't really care about it, and I'm on a budget regarding my server.  
Generate a token with `gen_token.sh` or `gen_token.ps1` and put it in the `config.conf` file on the server and the clients.
