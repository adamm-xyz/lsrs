# lsrs

`ls` rewritten in Rust. Lists directory contents, just like the original.

## Compile from Source

```sh
git clone https://github.com/adamm-xyz/lsrs.git
cd lsrs
cargo build
```

## Usage

`lsrs [options] [PATH]`

### Options

| Option           | Description                                                       | Default |
| ---------------- | ----------------------------------------------------------------- | ------- |
| -a, --all        | do not ignore entries starting with `.`                           | false   |
| -s, --sizes      | show sizes of files; use -h for human-readable units              | false   |
| --help           | show a help message                                              | false   |
| -h               | print sizes in human-readable units                               | false   |
| -r, --reverse    | reverse order when sorting (-S, -t)                               | false   |
| -S, --sort-size  | sort by file size, largest first (specify -r for smallest first)  | false   |
| -t, --sort-mtime | sort by time modified, newest first (specify -r for oldest first) | false   |

## Roadmap

- [x] Add colors
- [x] Add show hidden entries (-a)
- [x] Add show size of files in bytes (-s)
- [x] Add help message (--help)
- [x] Add human-readable option (-h)
- [ ] Add reverse order when sorting (-r)
- [ ] Add list subdirectories recursively (-R)
- [ ] Add sort by file size, largest first (-S)
- [ ] Add sort by time, newest first (-t)
- [ ] Add fill width with a comma separated list of entries (-m)
- [ ] Add long listing format (-l)
