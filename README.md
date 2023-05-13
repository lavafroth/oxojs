# oxojs

oxojs aims to be a drop-in replacement for `subjs` to fetch javascript files from a list of URLS or subdomains. Analyzing javascript files can help you find undocumented endpoints, secrets, and more.

It's recommended to pair this with @lc's [gau](https://github.com/lc/gau) and @GerbenJavado [https://github.com/GerbenJavado/LinkFinder](https://github.com/GerbenJavado/LinkFinder)

## Usage

One can either supply the path to an input file as the first argument

```sh
oxojs urls.txt
```

or pipe the URLs through standard input.

```sh
cat urls.txt | oxojs 
```

```sh
cat hosts.txt | gau | oxojs
```

Additionally, the following flags are displayed using the `--help` flag.

```sh
oxojs --help
```

| Flag | Description | Default value | Example value |
|------|-------------|---------------|---------------|
| `-c` | Number of concurrent workers to spawn | 4 | 40
| `-t` | Timeout (in seconds) for the client | 15 | 20
| `--user-agent` | User-Agent to send in requests | "oxojs/VERSION" | "Googlebot/2.1"
| `-o` | Filepath to write results to | `None` | "path/to/output"
| `--version` | Show version number | N/A | N/A


## Installation
### From Source

Install rust from rustup.rs or your package manager. Now run:

```sh
cargo install --git https://github.com/lavafroth/oxojs
```

### Binary Releases
Binary releases will be available once the project becomes stable.
