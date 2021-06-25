There is lot more to be done:

 - partial and full checksums for files
 - memory mapping of files (for direct comparison)
 - something better for comparing images (opencv perhaps)
 - multi-threading/asynchrony where possible

---------------------------
```
Author: Sarang Baheti, c 2021
Source: https://github.com/sarangbaheti/lsdups-rust
Usage: lsdups-rust [options]

Options:
    -d, --dir <DIRECTORY-PATH>
                        directory to traverse, defaults to current directory
    -p, --pattern <PATTERN>
                        pattern for files, defaults to all files
        --filter <SKIP-PATTERN>
                        pattern for files to filter out/skip, defaults to
                        empty-string
        --size <unsigned int>
                        filter all data before this size, defaults to 0
    -v, --verbose       version information and exit
    -h, --help          prints help

 ```
