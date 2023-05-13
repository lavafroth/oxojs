# oxojs

oxojs aims to be a drop-in replacement for `subjs` to fetch javascript files from a list of URLS or subdomains. Analyzing javascript files can help you find undocumented endpoints, secrets, and more.

It's recommended to pair this with @lc's [gau](https://github.com/lc/gau) and @GerbenJavado [https://github.com/GerbenJavado/LinkFinder](https://github.com/GerbenJavado/LinkFinder)

# Resources
- [Usage](#usage)
- [Installation](#installation)

## Usage:
Examples:
```sh
$ cat urls.txt | oxojs 
$ oxojs -i urls.txt
$ cat hosts.txt | gau | oxojs
```

To display the help for the tool use the `--help` flag:

```sh
$ oxojs --help
```


      --user-agent <USER_AGENT>    User-Agent to send in requests
  -o, --output <OUTPUT>            Filepath to write results to
  -c, --concurrency <CONCURRENCY>  Number of concurrent workers to spawn [default: 4]
  -t, --timeout <TIMEOUT>          Timeout (in seconds) for the client [default: 15]
  -h, --help                       Print help
  -V, --version                    Print version


One can either supply the path to an input file as the first argument or pipe
the URLs through standard input. Additionally, the following flags are
available.

| Flag | Description | Default value | Example value |
|------|-------------|---------------|---------------|
| `-c` | Number of concurrent workers to spawn | 4 | 40
| `-t` | Timeout (in seconds) for the client | 15 | 20
| `--user-agent` | User-Agent to send in requests | "oxojs/VERSION" | "Googlebot/2.1"
| `-o` | Filepath to write results to | `None` | "path/to/output"
| `--version` | Show version number | N/A | N/A


## Installation
### From Source:

This is the only installation method for now. Binary releases will be available once the project becomes stable.

```sh
cargo install --git https://github.com/lavafroth/oxojs
```
