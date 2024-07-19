# Custom Binary Format for packing multiple files together because I don't want to use tar

## How does it work?

Well it's simple, the file structure is as follows:

- missing files size (2 bytes)
- missing file name size (1 byte) - if missing files size is 0 then this is not present
- missing file name (missing file name size bytes) - same as above
- file size (4 bytes)
- file name size (1 byte)
- file name (file name size bytes)
- file content (file size bytes)

As you can see the names of the missing files (files that the client has that the server doesn't) are also stored in the binary file. This is done to notify to the client the files it needs to send to server and make at least one less request to the server.
