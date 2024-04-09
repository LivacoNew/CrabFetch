# CrabFetch
CrabFetch is a highly performant and customisable command-line fetch tool. Like all others it grabs system information and displays it in a fancy way.<br>
What makes CrabFetch different is that _it aims to be as performant as possible_, unlike most other popular fetch programs. That way, when you start up your riced terminal with your fetcher at the top, you no longer need to feel angry it takes that extra 0.05s to load in!

![A preview image of CrabFetch.](https://i.imgur.com/2gyTObl.png)


## Performance
As it's CrabFetch's main selling point, here's a table comparing performance between [NeoFetch](https://github.com/dylanaraps/neofetch), [FastFetch](https://github.com/fastfetch-cli/fastfetch) and CrabFetch.<br>
All benchmarks were done via the zsh implemented `time` keyword, with as similar configs as possible.

| **Fetcher**     | **Time Ran**                                                                              |
| --------------- | ----------------------------------------------------------------------------------------  |
| ***CrabFetch*** | [crabfetch  0.01s user 0.00s system 92% cpu 0.006 total](https://i.imgur.com/2n5ozXH.png) |
| FastFetch       | [fastfetch  0.05s user 0.01s system 97% cpu 0.056 total](https://i.imgur.com/yPrCsEi.png) |
| NeoFetch        | [neofetch  0.07s user 0.07s system 80% cpu 0.172 total](https://i.imgur.com/5J2KE9m.png)  |

<sub>NOTE: This is with the "pcisysfile" GPU method without caching. For more details see the FAQ under "Does CrabFetch cheat with it's performance?"</sub>


## Installation
> [!IMPORTANT]
> Only Linux based OS's are supported. Any other operating system will not work.
> 
Check out the [Wiki Page](https://github.com/LivacoNew/CrabFetch/wiki/Installation) for how to install CrabFetch.

## Configuration
To generate the default configuration file, run `crabfetch -g`.<br>
The configuration file should be in `~/.config/CrabFetch/config.toml`. From there, refer to either the comments or the [wiki page](https://github.com/LivacoNew/CrabFetch/wiki/Configuration).


## Not got what you want?
Make a issue and I'll see if I can add it in. I'm aware support is kind of sparse right now, as I still need to test on different types of systems.


## Credits
- [NeoFetch](https://github.com/dylanaraps/neofetch) for being a occasional source on where to find some info in Linux.
- [FastFetch](https://github.com/fastfetch-cli/fastfetch) Also a source for system info.


## FAQ
### Does CrabFetch cheat with it's performance?
Not by default however it is reccomended to enable it.<br>
The only problem comes in with GPU info, and how slow it is to find. CrabFetch provides two ways to get it;
- By scanning `/sys/bus/pci/devices` for any GPU's and parsing from there.
- By using `glxinfo`, which provides the info directly.

The first method is eons faster than waiting for `glxinfo`, however `glxinfo` is more accurate. E.g, [the first method](https://i.imgur.com/IzWCnlF.png) gives my gpu as either a RX 7700 or RX 7800 whereas [glxinfo](https://i.imgur.com/k7ds3ZK.png) gets it bang on as a 7800.<br>
If you don't believe how slow it is, trust me, [glxinfo is slow...](https://i.imgur.com/YlzENV4.png)

To allow for accurate GPU info with good performance, _CrabFetch allows you to select to cache the GPU info_. For the sys file method this is kind of useless but for glxinfo this means you can have the full accuracy with the same performance as parsing sys files.

Ultimately, this is up to the user to select. **By default, CrabFetch will use sys files without caching**. While I highly recommend using the cached info, I won't set it to be the default so as to not be unfair to other fetchers with performance comparisons.

### So how does CrabFetch get it's performance in everything else aside from the GPU?
Honestly, aside from using Rust (hence the name) and trying not to reinvent the wheel it's just me programming with a keen eye on the runtime duration. Not really much else too it.

### Why make this in the first place?
I was fed up of NeoFetch having to load in every time I spawned a terminal. FastFetch is fine enough with performance however I find it's config really unintuitive and in general it's just kind of yucky. Hence CrabFetch was born to try to solve both problems.

### Do you plan on supporting other operating systems?
Not anytime soon. I already work on limited time for all my projects, and having to boot up VM's to test beta software constantly is annoying. That plus the idea of working with Windows again scares me.


## Working On (Roadmap)
- [ ] [Kitty Image Support](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
- [ ] Multiple GPUs
- [ ] Local IP
- [ ] Music Player / Song
