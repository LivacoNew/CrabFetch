# CrabFetch
CrabFetch is a fetch tool that fetches system information and displays it in a fancy way. Only Linux is supported.
Unlike most popular tools, CrabFetch is written in Rust to stay as performant as possible.

![A preview image.](https://i.imgur.com/dJTl6SU.png)

## Customisation
The customisation file should be in `~/.config/CrabFetch/config.toml`. The default configuration can be found in `default-config.toml` in this repo.

## Performance
Here's a table comparing performance between [NeoFetch](https://github.com/dylanaraps/neofetch), [FastFetch](https://github.com/fastfetch-cli/fastfetch) and CrabFetch.

All benchmarks were done via the zsh implemented `time` command, with as similar configs as possible.

| **Fetcher** | **Time Ran**                                                | **Image**                       |
| ----------- | ----------------------------------------------------------- | ------------------------------- |
| CrabFetch   | ./crabfetch  0.00s user 0.00s system 93% cpu 0.003 total    | https://i.imgur.com/iuQGqiz.png |
| NeoFetch    | /bin/neofetch  0.08s user 0.04s system 104% cpu 0.111 total | https://i.imgur.com/jdwvdY5.png |
| FastFetch   | /bin/fastfetch  0.00s user 0.01s system 85% cpu 0.008 total | https://i.imgur.com/Z7Pm5oA.png |

## Installation
From source;
```sh
git clone git@github.com:LivacoNew/CrabFetch.git
cd CrabFetch
cargo install --path .
```
Ensure you've got your $PATH set to include ~/.cargo/bin/

## Credits
- [NeoFetch](https://github.com/dylanaraps/neofetch) for being a occasional source on where to find hardware info in Linux.

## Roadmap
- [ ] Displays
- [ ] Battery
- [ ] Host
- [ ] Packages
- [x] Command line arguments for Version & Custom config
- [ ] Image Support
- [ ] Config Cleanup
- [ ] Config Defaults
