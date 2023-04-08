# A Linux Program Launcher
This is a Program Launcher written in Rust.
For the moment it uses multiple different UI toolkits, namely:
- egui 
- \TODO iced
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

