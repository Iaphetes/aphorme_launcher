# A Linux Program Launcher
This is a Program Launcher written in Rust.
For the moment it uses multiple different UI toolkits, namely:
- [x] egui 
- ~~ [ ] iced~~ While this seems to be a good toolkit, it (similarly to egui) is not designed for use in floating launchers or widgets.
I will instead have a look at gtk in Rust and other toolkits. Any suggestions are always welcome.
# Features
- Searches for all desktop files in most of the common linux application paths
  - Can be extended using the config file (See `Configuration->app_cfg->paths`)
- Can be used for dmenu type selection of piped in applications
# Why multiple UI toolkits?
I started using egui for this, which is a great toolkit, but it has some limitations. So I thought I'd try some other toolkits. This I think is a reasonably simple UI, but with some quirks (like the keyboard scrolling) which I can use to compare ease of use and feature sets.
# Why another program launcher?
There are several reasons I wanted a new program launcher:
## 1. Wayland **and** X11 compatibility
Most good program launchers like Rofi, Fuzzel etc. only support either X11 or wayland. Since I still sometimes switch between X11 and Wayland I wanted a launcher, that ran on both systems
## 2. Wayland clones don't always work
There are some cool projects like Wofi that aim to port X11 programs to Wayland. However I found at least Wofi to have some bugs with my window setup.
## 3. I wanted a project that can be reasonably finished
I tend to take on huge projects which never end. I saw this as a comparatively simple project.
# Configuration
Configuration is now found in $HOME/.config/aphorme/config.toml
## gui_cfg
Options for the gui.
### icons: boolean
Enable or disable icon loading.
### GuiFramework
Which GuiFramework to used. At the moment EGUI and ICED.
Note that at the moment ICED is not compiled into the launcher by default. To do so compile with the feature `iced-ui`
## app_cfg
App spanning options.
### paths
List of additional paths to search. Home directory can only be denoted by using `$HOME`
Defaults to an empty list.
### use_default_paths
Search default paths. If paths is defined appends them to the default.
Defaults to `true`

Default paths are
```toml
  "/usr/share/applications",
  "/usr/local/share/applications",
  "$HOME/.local/share/applications",
  "/var/lib/flatpak/exports/share/applications"
```
### preferred_apps
Contains configuration for the preferred apps (aka. the last used apps)
#### max_weight
Maximum weight allowed.
Defaults to 10.
## Example Config
```toml
[gui_cfg]
icon = true
ui_framework = 'EGUI'
[app_cfg]
paths = ["$HOME/Desktop"]
```

# Known issues
## Scrolling is somewhat weird (and the scroll bar is visible) in egui
The egui ScrollArea does not allow for movement using the arrow keys. This means I had to implement that myself. The method I chose (just remembering the index) does however overwrite the scrolling using the mousewheel/touchpad gestures etc. This means I had to implement the scrolling with the scrollwheel myself, which 'fights' against the default scrolling. This causes minor visual glitches but so far no actual bugs
# Changes
## 0.1.11
### Features added
- Custom selection of inputs now possible. Will echo the selected option (similar to dmenu)
## 0.1.12
### Fixes
- For some reason sway seems to pass whitespace to a launched program using exec. This lead the list to be empty
## 0.1.13
### Features added
- Right(down) and left(up) arrow keys added to list navigation
### Fixes
- Fixes stdin thread panicing. Now it shuts down gracefully, once there is no input anymore
## 0.1.14
### Features added
- New "Preferred Apps" feature, which remembers frequently used apps and puts them on top in searches.
### Fixes
- Cleaned up the code somewhat
- The code will now not error out, when a Desktop file is not readable. If you are missing desktop files run the launcher in the terminal and check the error output.
