# lsrs

`ls` rewritten in Rust. Lists directory contents, just like the original.

## Usage

`target/debug/lsrs [options] [PATH]`

### Options

| Option      | Description                             | Default |
| ----------- | --------------------------------------- | ------- |
| -a, --all   | do not ignore entries starting with `.` | false   |
| -s, --sizes | show sizes of files in bytes            | false   |
| -h, --help  | print this help message                 |         |
