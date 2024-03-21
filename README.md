# CrabFetch
CrabFetch is a fetch tool that fetches system information and displays it in a fancy way. Only Linux is supported.
Unlike most popular tools, CrabFetch is written in Rust to stay as performant as possible.

![A preview image.](https://i.imgur.com/dJTl6SU.png)

## Customisation
The customisation file should be in `~/.config/CrabFetch/config.toml`. The default configuration can be found in `default-config.toml` in this repo.

## Performance
Here's a table comparing performance between [NeoFetch](https://github.com/dylanaraps/neofetch), [FastFetch](https://github.com/fastfetch-cli/fastfetch) and CrabFetch.

| **Fetcher** | **Time Ran**                                 | **Image**                       |
| ----------- | -------------------------------------------- | ------------------------------- |
| CrabFetch   | 0.02s user 0.01s system 48% cpu 0.065 total  | https://i.imgur.com/9GkW1LK.png |
| NeoFetch    | 0.11s user 0.08s system 101% cpu 0.188 total | https://i.imgur.com/ywWkoJm.png |
| FastFetch   | 0.09s user 0.02s system 99% cpu 0.108 total  | https://i.imgur.com/dJTl6SU.png |

## To-Do
- [x] Swap Space
- [x] GPU
- [ ] Displays
- [ ] Battery
- [ ] Host
- [ ] Packages
- [ ] Command line arguments for Version & Custom config
- [x] Check for running Linux + All required commands are indeed present
- [x] Rename all variables to include data types

## Credits
- [NeoFetch](https://github.com/dylanaraps/neofetch) for being a good source on where to find hardware info in Linux.
