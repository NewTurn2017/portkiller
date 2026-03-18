# pk — Port Killer

Kill development server ports instantly.

```
pk              # Interactive multi-select
pk 3000         # Kill port 3000 directly
pk 3000 8080    # Kill multiple ports
pk -l           # List listening ports
pk -a           # Kill all listening ports
```

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/NewTurn2017/portkiller/main/install.sh | sh
```

## Requirements

- macOS or Linux
- `lsof` (pre-installed on macOS, available on most Linux distros)

## Build from source

```bash
git clone https://github.com/NewTurn2017/portkiller.git
cd portkiller
cargo build --release
cp target/release/pk /usr/local/bin/
```

## License

MIT
