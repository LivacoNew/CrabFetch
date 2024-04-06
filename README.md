# CrabFetch
CrabFetch is a command-line fetch tool, which like all others grabs system information and displays it in a fancy way. What makes it different is that _it aims to be as performant as possible_, unlike most other popular fetch programs. That way, when you start up your riced terminal with your fetcher at the top, you no longer need to feel angry it takes 0.5s to load in!

![A preview image of CrabFetch.](https://i.imgur.com/sP5cgm2.png)


## Performance
As it's CrabFetch's main selling point, here's a table comparing performance between [NeoFetch](https://github.com/dylanaraps/neofetch), [FastFetch](https://github.com/fastfetch-cli/fastfetch) and CrabFetch.

All benchmarks were done via the zsh implemented `time` keyword, with as similar configs as possible.

| **Fetcher**     | **Time Ran**                                           | **Image**                       |
| --------------- | ------------------------------------------------------ | ------------------------------- |
| NeoFetch        | neofetch  0.08s user 0.06s system 93% cpu 0.143 total  | https://i.imgur.com/BHOhpii.png |
| FastFetch       | fastfetch  0.04s user 0.02s system 99% cpu 0.053 total | https://i.imgur.com/vMPXJF0.png |
| ***CrabFetch*** | crabfetch  0.00s user 0.00s system 95% cpu 0.005 total | https://i.imgur.com/82SChkA.png |

<sub>NOTE: This is slightly unfair. See the FAQ under "Does CrabFetch cheat with it's performance?"</sub>


## Installation
> [!IMPORTANT]
> Only Linux based OS's are supported. Any other operating system will not let you run CrabFetch.

### Dependencies
These are only required if you are using their associated modules.
- The `df` command, used for getting mounted disks.
- The `glxinfo` command. Only required for the "glxinfo" GPU method.
- `pciutils` package, specifically only for the `pci.ids` file. Only required for the "pcisysfile" GPU method.

### From Source
```sh
git clone git@github.com:LivacoNew/CrabFetch.git
cd CrabFetch
cargo install --path .
```
Ensure you've got your $PATH set to include ~/.cargo/bin/


## Configuration
The configuration file should be in `~/.config/CrabFetch/config.toml`. The default configuration can be found in `default-config.toml` in this repo, and you may have to copy and paste it over. From there, the comments should keep you right.


## Not got what you want?
Make a issue and I'll see if I can add it in. I'm aware support is kind of sparse right now, as I still need to test on different types of systems.


## Credits
- [NeoFetch](https://github.com/dylanaraps/neofetch) for being a occasional source on where to find some info in Linux.
- [FastFetch](https://github.com/fastfetch-cli/fastfetch) Also a source for system info.


## FAQ
### Does CrabFetch cheat with it's performance?
It can, but not by default. The problem comes with GPU info, and how slow it is to find. CrabFetch provides two ways to get it;
- By scanning `/sys/bus/pci/devices` for any GPU's and parsing from there.
- By using `glxinfo`, which provides the info directly.
The first method is eoens faster than waiting for `glxinfo`, however `glxinfo` is more accurate. E.g, [the first method](https://i.imgur.com/IzWCnlF.png) gives my gpu as either a RX 7700 or RX 7800 whereas [glxinfo](https://i.imgur.com/k7ds3ZK.png) gets it bang on as a 7800.
To allow for accurate GPU info with good performance, CrabFetch allows you to select to cache the GPU info. For the sys file method this is kind of useless but for glxinfo this means you can have the full accuracy with the same performance as parsing sys files.
Ultimately, this is up to the user to select. **By default, CrabFetch will use sys files without caching**. While I highly recommend using the cached info, I won't set it to be the default so as to not be unfair to other fetchers with performance comparisons.

### So how does CrabFetch get it's performance in everything else aside from the GPU?
To start, CrabFetch is written in Rust (hence the name). On top of that is tries to prevent re-inventing the wheel and tries to use whatever Linux has done for it already e.g using stuff in /sys or in the environment variables.
Honestly, that's about it aside from me programming with a keen eye on the runtime duration.

### Do you plan on supporting other operating systems?
Not anytime soon. I already work on limited time for all my projects, and having to boot up VM's to test beta software constantly is annoying. That plus the idea of working with Windows again scares me.


## Working On (Roadmap)
- [ ] Battery Module
- [ ] Segment Modules
- [ ] Kitty Image Support
- [ ] Fix decimal places
- [x] Non-Cached GPU
- [ ] Multiple GPUs
- [ ] Config Cleanup
- [x] Mute errors parameter
