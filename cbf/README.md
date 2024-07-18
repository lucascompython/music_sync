# Custom Binary Format for packing multiple files together because I don't want to use tar

## How does it work?

Well it's simple, for each file, we write a header that contains the file size, file name size, file name, then we write the file content.
The header is 4 bytes for the file size and 1 byte for the file name length, then the file name.
