# CrabFetch
[![Packaging status](https://repology.org/badge/tiny-repos/crab-fetch.svg)](https://repology.org/project/crab-fetch/versions)
[![latest packaged version(s)](https://repology.org/badge/latest-versions/crab-fetch.svg)](https://repology.org/project/crab-fetch/versions)

CrabFetch is a highly performant and extremelty easily customisable command-line fetch tool. Like all others it grabs system information and displays it in a fancy way.<br>
What makes CrabFetch different is that _it aims to be as performant as possible_ while still remaining easy to use. That way, when you start up your riced terminal with your fetcher at the top, you no longer need to feel angry it takes that extra 0.05s to load in!

![Showcase 1](https://i.imgur.com/k6GMh63.png)
![Showcase 2](https://i.imgur.com/abu7XOl.png)


## Performance
Comparing the performance of; [NeoFetch](https://github.com/dylanaraps/neofetch), [FastFetch](https://github.com/fastfetch-cli/fastfetch) and CrabFetch.  
Configs were created to match up as closely as possible, with these benchmarks being taken by `hyperfine 1.18.0`.  
![As close as possible configs](https://i.imgur.com/cxwm7I5.png)  
  
The following screenshot shows the results;  
![CrabFetch runs the fastest.](https://i.imgur.com/ZvJGw7H.png)  
  
Try this benchmark yourself, if you don't get as good performance please [make a performance issue](https://github.com/LivacoNew/CrabFetch/issues/new?assignees=&labels=performance&projects=&template=performance-issue.md&title=) and let me know!

More in-depth benchmarks can be found [on the wiki](https://github.com/LivacoNew/CrabFetch/wiki/Benchmarks).


## System Support
If CrabFetch doesn't correctly detect something on your system, [make a issue](https://github.com/LivacoNew/CrabFetch/issues/new?assignees=&labels=detection&projects=&template=detection-issue.md&title=) and I'll go hunting for it!


## Installation
> [!IMPORTANT]
> Only Linux based OS's are supported. Any other operating system will not work.
>
[![Packaging status](https://repology.org/badge/vertical-allrepos/crab-fetch.svg)](https://repology.org/project/crab-fetch/versions)

Check out the [Wiki Page](https://github.com/LivacoNew/CrabFetch/wiki/Installation) for more detailed instructions on how to install CrabFetch, as well as manually building.

### Arch Linux
You can use either;
- [crab-fetch](https://aur.archlinux.org/packages/crab-fetch) (AUR)
- [crab-fetch-git](https://aur.archlinux.org/packages/crab-fetch-git) (AUR)

### Debian
.deb files are provided in releases as of `0.3.0`. From there, simply install it using;
```sh
sudo apt install ./crabfetch.deb
```

### Other
Go to the [latest release](https://github.com/LivacoNew/CrabFetch/releases/latest) and download the file for your CPU's architecture. From there, just run;
```sh
cp crabfetch /usr/local/bin/crabfetch
```
**Be aware that this means your package manager will not be aware of CrabFetch.**

## Configuration
To generate the default configuration file, run `crabfetch -g`.<br>
The configuration file should be in `~/.config/CrabFetch/config.toml`. From there, refer to either the comments or the [wiki page](https://github.com/LivacoNew/CrabFetch/wiki/Configuration).

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
