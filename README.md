# Music Sync

It's a simple tool that I made to sync my music library between my devices.

## How does it work?

There is a server that hosts the music library and clients that sync the music library with the server.  
The clients may also update the music library on the server.

## Security

Because the clients can update the music library on the server and thus write an important amount of data, there is a token authentication system.  
Keep in mind that the data is not encrypted, because I don't really care about it, and I'm on a budget regarding my server.  
Generate a token with `gen_token.sh` or `gen_token.ps1` and put it in the `config.conf` file on the server and the clients.

## Things

For now the server always has the files loaded into memory. It can be a problem if the music library is too big. But can also be a good thing because, for example, my server is a 2009 laptop with a slow HDD.  
I'll probably add a way to load the files on demand.

I'll also benchmark packing the files together vs keeping them separated and having the client send multiple requests in parallel.

And I'll add compression to the files, probably with `zstd` or `lz4`.
