# PNG Steganography

Encode a hidden message within a png file using LSB Steganography.
The size of the hidden message in bytes should be no greater than 

``` python
min(2^32 - 1, size(png_file) / 8)
```

One bit of the hidden message is encoded for every byte per pixel of the png file.

### Usage:

To encode an a message using the base `image.png` into `encoded.png`,
you can pipe the message to stdin

``` bash
echo "Hidden message" | cargo run -e image.png encoded.png
```
or you can pass a `file-to-encode.bin`

``` bash
cargo run -- -e image.png encoded.png file-to-encode.bin
```
It doesn't matter what format the data is in.

To decode a message, you can pipe the decoded message to stdout

``` bash
cargo run -- -d encoded.png >> decoded-file.bin
```
or you can write the decoded message to a file

``` bash
cargo run -- -d encoded.png decoded-file.bin
```

