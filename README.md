# CrabFetch
CrabFetch is a command-line fetch tool, which like all others grabs system information and displays it in a fancy way. What makes it different is that _it aims to be as performant as possible_, unlike most other popular fetch programs. That way, when you start up your riced terminal with your fetcher at the top, you no longer need to feel angry it takes 0.5s to load in!

![A preview image of CrabFetch.](https://i.imgur.com/dJTl6SU.png)


## Performance
As it's CrabFetch's main selling point, here's a table comparing performance between [NeoFetch](https://github.com/dylanaraps/neofetch), [FastFetch](https://github.com/fastfetch-cli/fastfetch) and CrabFetch.

All benchmarks were done via the zsh implemented `time` keyword, with as similar configs as possible.

| **Fetcher**   | **Time Ran**                                           | **Image**                       |
| ------------- | ------------------------------------------------------ | ------------------------------- |
| NeoFetch      | neofetch  0.06s user 0.04s system 92% cpu 0.106 total  | https://i.imgur.com/AoExg0s.png |
| FastFetch     | fastfetch  0.04s user 0.01s system 98% cpu 0.051 total | https://i.imgur.com/Q9csdAo.png |
| **CrabFetch** | crabfetch  0.00s user 0.00s system 92% cpu 0.003 total | https://i.imgur.com/iuQGqiz.png |

<sub>NOTE: This is slightly unfair. See the FAQ under "Does CrabFetch cheat with it's performance?"</sub>


## Installation
> [!IMPORTANT]
> Only Linux based OS's are supported. Any other operating system will not let you run CrabFetch.

### Dependencies
These are only required if you are using their associated modules.
- The `df` command, used for getting mounted disks.
- The `glxinfo` command, used for getting GPU info.

### From Source
```sh
git clone git@github.com:LivacoNew/CrabFetch.git
cd CrabFetch
cargo install --path .
```
Ensure you've got your $PATH set to include ~/.cargo/bin/


## Configuration
The configuration file should be in `~/.config/CrabFetch/config.toml`. The default configuration can be found in `default-config.toml` in this repo, and you may have to copy and paste it over. From there, the comments should keep you right.


## Credits
- [NeoFetch](https://github.com/dylanaraps/neofetch) for being a occasional source on where to find some info in Linux.
- [FastFetch](https://github.com/fastfetch-cli/fastfetch) Also a source for system info.


## FAQ
##### Does CrabFetch cheat with it's performance?
Kinda. Everything is done at runtime, except from GPU info. This is because `glxinfo` which I use to get GPU info is stupidly slow, so I cache the parts I need at `/tmp/crabfetch-gpu` since no one's going to be swapping in/out GPUs. I have a plan to get rid of this cache and make it's performance real, but for now the answer is yes.

Using the --ignore-cache flag, the true performance can be seen to run [at 0.062s](https://i.imgur.com/igv4ZyG.png), coming in just behind FastFetch on the performance table above.

##### So how does CrabFetch get it's performance in everything else aside from the GPU?
To start, CrabFetch is written in Rust (hence the name). On top of that is tries to prevent re-inventing the wheel and tries to use whatever Linux has done for it already e.g using stuff in /sys or in the environment variables.
Honestly, that's about it aside from me programming with a keen eye on the runtime duration.

##### Do you plan on supporting other operating systems?
Not anytime soon. I already work on limited time for all my projects, and having to boot up VM's to test beta software constantly is annoying. That plus the idea of working with Windows again scares me.


## Working On (Roadmap)
- [ ] Battery
- [ ] Custom Modules
- [ ] Image Support
- [ ] Config Cleanup
- [ ] Config Defaults
