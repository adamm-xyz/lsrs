# lsrs

`ls` rewritten in Rust. Lists directory contents, just like the original.

## Compile from Source

```
git clone https://github.com/adamm-xyz/lsrs.git
cd lsrs
cargo build
```

## Usage

`./target/debug/lsrs [options] [PATH]`

### Options

| Option      | Description                             | Default |
| ----------- | --------------------------------------- | ------- |
| -a, --all   | do not ignore entries starting with `.` | false   |
| -s, --sizes | show sizes of files in bytes            | false   |
| -h, --help  | print this help message                 |         |

## Roadmap
- [x] Add colors
- [x] Add show hidden entries (-a)
- [x] Add show size of files in bytes (-s)
- [x] Add help message (--help)
- [x] Add human-readable option (-H)
- [ ] Add reverse order when sorting (-r)
- [ ] Add list subdirectories recursively (-R)
- [ ] Add sort by file size, largest first (-S)
- [ ] Add sort by time, newest first (-t)
- [ ] Add fill width with a comma separated list of entries (-m)
- [ ] Add long listing format (-l)

