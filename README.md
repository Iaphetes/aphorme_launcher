# A Linux Program Launcher
This is a Program Launcher written in Rust.
For the moment it uses multiple different UI toolkits, namely:
- [x] egui 
- [ ] iced
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
## 0.1.3 
### Features
- A config file was added (see Configuration). At the moment only icon loading is configured here.
### Fixes
- Launcher is forced into floating mode on tiling window managers to work like rofi etc.

## 0.1.4 
### Fixes
- Changed the default for icons. Previously they were disabled by default now the default is enabled.
If you want to change that see the Configuration part in this README
## 0.1.5
### Fixes
- Improved startup time by loading icons in batches of 5 at every repaint of the UI
## 0.1.6
### Fixes
- Now checks if it is the only instance and if not, it will close down
- Removed parallel iterator from loading, since the overhead was too great
- Made code more readable (Thanks AnyTimeTraveler)
- Started work on iced (not yet usable)
## 0.1.7
### Fixes
- Removed unused dependencies
- Fixed bugs where a combination of features could lead to crashes
## 0.1.8
### Fixes
- Fixed bug where the socket opened by a library would not be closed after running a program
## 0.1.9
### Fixes
- Fixed bug where one could not scroll past the 100th item in the search (silly me for leaving that hardcoded from my tests...)
- Made compilation not dependant on nightly toolchain
- Fixed config bug, where a non-existent config led to a crash. Now the config will always be created in $HOME/aphorme/config.toml
## 0.1.10
### Features added
- You can now add additional paths to search or replace them entirely. See the `Config` section for more on this.
### Fixes
- Now properly resolvec `$HOME` to the home directory