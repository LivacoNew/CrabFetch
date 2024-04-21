# CrabFetch
CrabFetch is a highly performant and customisable command-line fetch tool. Like all others it grabs system information and displays it in a fancy way.<br>
What makes CrabFetch different is that _it aims to be as performant as possible_ while still remaining easy to use. That way, when you start up your riced terminal with your fetcher at the top, you no longer need to feel angry it takes that extra 0.05s to load in!

![Showcase 1](https://i.imgur.com/MTAULWp.png)
![Showcase 2](https://i.imgur.com/c6S2q9n.png)
![Showcase 3](https://i.imgur.com/jz2ioEH.png)
![Showcase 4](https://i.imgur.com/kJ7l93H.png)


## Performance
Below is a table comparing performance between [NeoFetch](https://github.com/dylanaraps/neofetch), [FastFetch](https://github.com/fastfetch-cli/fastfetch) and CrabFetch.<br>
All benchmarks were done via the [hyperfine](https://archlinux.org/packages/extra/x86_64/hyperfine/) utility, on the same machine. Configs were created to match up as closely as possible.

**Please note you may not get the same times**, depending on what modules you have enabled and depending on your system. From testing, NeoFetch is dead in the dust but FastFetch and CrabFetch are typically either neck and neck or one wins over the other. I'm already looking into more ways to improve CrabFetch's times, but it depends on your environment/needs which one is actually a better fit. For all of my personal systems however, CrabFetch wins.

![Benchmark Results](https://i.imgur.com/6INKbtk.png)

## System Support
Currently, support for different softwares on CrabFetch is limited from me not having the time to go out and implement everything required. I've tried to implement all the most common ones but ultimately I will miss some.
A full list of what is supported can be found on the [Wiki](https://github.com/LivacoNew/CrabFetch/wiki/Compatability-List).
If CrabFetch doesn't support what you use, **make a issue** and I'll go about it!


## Installation
> [!IMPORTANT]
> Only Linux based OS's are supported. Any other operating system will not work.
>
Check out the [Wiki Page](https://github.com/LivacoNew/CrabFetch/wiki/Installation) for detailed instructions on how to install CrabFetch, as well as manually building.

### Arch Linux
You can use either;
- [crab-fetch](https://aur.archlinux.org/packages/crab-fetch) (AUR)
- [crab-fetch-git](https://aur.archlinux.org/packages/crab-fetch-git) (AUR)

### Other
Go to the [latest release](https://github.com/LivacoNew/CrabFetch/releases/latest) and download the file for your CPU's architecture. From there, just run;
```sh
cp crabfetch-x86-64 /usr/local/bin/crabfetch
```
**Be aware that this means your package manager will not be aware of CrabFetch.**

## Configuration
To generate the default configuration file, run `crabfetch -g`.<br>
The configuration file should be in `~/.config/CrabFetch/config.toml`. From there, refer to either the comments or the [wiki page](https://github.com/LivacoNew/CrabFetch/wiki/Configuration).


## Not got what you want?
Make a issue and I'll see if I can add it in. I'm aware support is kind of sparse right now, as I still need to test on different types of systems.


## Credits
- [NeoFetch](https://github.com/dylanaraps/neofetch) for being an occasional source on where to find some info in Linux.
- [FastFetch](https://github.com/fastfetch-cli/fastfetch) Also a source for system info, as well as the the author informing me of numerous issues I was unaware of.


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
- [ ] Config Presets
- [ ] RAM Speed
- [ ] Local IP
- [ ] Music Player / Song
