# gpmf

## Parser and Writer for GoPro Metadata Format (GPMF)

WIP: Currently successfully parses all raw test data and logs the results.

## Design Goals

* Linux Philosophy, each tool does one thing and does it well
* Can be integrated in other tools, and in other languages than Rust
* Focus on clean, easy to understand code (no macros)
* Performant (but not at the expense of the previous item)
* Easy to read detailed log in order to be able to debug problems
* Memory safe parser
* Zero security vulnerabilities. Avoid problems found in other tools e.g.: [GoPro GPMF-parser Vulnerabilities](https://blog.inhq.net/posts/gopro-gpmf-parser-vuln-1/)
* Never generate exceptions, i.e.: Should never panic.
* Should pass fuzz tests i.e.: handle junk data
* Should avoid DOS attacks. Possibly Add max buffer lengths.
* Gracefully recover from errors
* Handle unknown tags

## Reporting Issues

If you have a file that is not handled please submit an issue, attaching the raw metadata file

## Feature Roadmap

* [x] Parser (WIP) at present just prints out data
* [ ] Create a structure to hold data
* [ ] Handle Scale
* [ ] Handle multiple sensor data 'mp4 boxes/atoms', as contained in mp4 file
* [ ] Return data in chronological order using Iterator and Tournament Tree
* [ ] Stream data
* [ ] Handle image exif data
* [ ] Writer

License: MIT OR Apache-2.0
