# CrabFetch
[![Packaging status](https://repology.org/badge/tiny-repos/crab-fetch.svg)](https://repology.org/project/crab-fetch/versions)
[![latest packaged version(s)](https://repology.org/badge/latest-versions/crab-fetch.svg)](https://repology.org/project/crab-fetch/versions)

CrabFetch is a highly performant and extremely easily customisable command-line fetch tool. Like all others it grabs system information and displays it in a fancy way.<br>
What makes CrabFetch different is that _it aims to be as performant as possible_ while still remaining easy to use. That way, when you start up your riced terminal with your fetcher at the top, you no longer need to feel angry it takes that extra 0.05s to load in!

![Showcase 1, CrabFetch with all it's modules enabled.](https://i.imgur.com/pAOwyEC.png)
![Showcase 2, CrabFetch with a more sensible config](https://i.imgur.com/zr9x8l8.png)

**Do note that CrabFetch is quite early in it's life and shouldn be considered in "beta". You may encounter issues.**  
**Please, report _ALL_ issues and help me improve it. You may even have fun breaking it!**


## Performance Showcase
We'll compare the performance of; [NeoFetch](https://github.com/dylanaraps/neofetch), [FastFetch](https://github.com/fastfetch-cli/fastfetch) and CrabFetch.  
Configs were created to match up as closely as possible, with these benchmarks being taken by `hyperfine 1.18.0`.  
![Screenshot showing each fetch, running with as close of a configs as possible given it's features](https://i.imgur.com/kWafK3J.png)  

<sub>**NOTE 1:** NeoFetch did not let me disable my CPU's integrated GPU, so it is the only one that displays the "Raphiel" GPU. While CrabFetch can be toggled to display it, I could't find a way to tell FastFetch to, so I opted to leave it as the odd one out.</sub><br>
<sub>**NOTE 2:** NeoFetch also does not find my `/hdd` mount.</sub>
  
The following screenshot shows the results;  
![Screenshot showing CrabFetch runs the fastest.](https://i.imgur.com/2rezkQv.png)  
  
Try this benchmark yourself! If you don't get as good performance please [make a performance issue](https://github.com/LivacoNew/CrabFetch/issues/new?assignees=&labels=performance&projects=&template=performance-issue.md&title=) and let me know so I can investigate.

More in-depth benchmarks can be found [on the wiki](https://github.com/LivacoNew/CrabFetch/wiki/Benchmarks).


## System Support
CrabFetch is very early in it's life, and may not detect some stuff correctly, there's no "standard" for fetching information across every system! If it doesn't detect something on your system, [make a issue](https://github.com/LivacoNew/CrabFetch/issues/new?assignees=&labels=detection&projects=&template=detection-issue.md&title=) so I can go hunting for it!


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
- [FastFetch](https://github.com/fastfetch-cli/fastfetch) An occasional source on where to find info in Linux, as well as it's author being extremely helpful in the repository.
- [NeoFetch](https://github.com/dylanaraps/neofetch) Another occasional source on where to find info.


## FAQ
### Does CrabFetch cheat with it's performance (e.g Caching info in the background)?
No.<br>

### Is CrabFetch stable?
Kind of. It's a hell of a lot more stable than it previously was, but should still be considered Alpha software. This isn't because CrabFetch is broken but simply because support for different systems is still small. Please help out by making issues and complaining at me to fix them!

### Why should I use this?
I think that's best answered by why I made it in the first place; I was fed up of NeoFetch having to load in every time I spawned a terminal, and while FastFetch had the performance, I found it's setup and usage quite unintuitive. Hence CrabFetch was born to try to solve both problems.

### Do you plan on supporting other operating systems other than Linux?
Not anytime soon, the idea of working with Windows again scares me and I only use Linux so I don't really have a reason to.

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=livaconew/crabfetch&type=Date&theme=dark)](https://star-history.com/#livaconew/crabfetch&Date)
