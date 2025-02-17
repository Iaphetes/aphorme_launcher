# 0.1.11

## Features added

- Custom selection of inputs now possible. Will echo the selected option (similar to dmenu)

# 0.1.12

## Fixes

- For some reason sway seems to pass whitespace to a launched program using exec. This lead the list to be empty

# 0.1.13

## Features added

- Right(down) and left(up) arrow keys added to list navigation

## Fixes

- Fixes stdin thread panicing. Now it shuts down gracefully, once there is no input anymore

# 0.1.14

## Features added

- New "Preferred Apps" feature, which remembers frequently used apps and puts them on top in searches.

## Fixes

- Cleaned up the code somewhat
- The code will now not error out, when a Desktop file is not readable. If you are missing desktop files run the launcher in the terminal and check the error output.


# 0.1.15

## Breaking changes

- Direct input piping now requires the --select-from-stdin argument

## Fixes

- Delay before stdin handling was too short. Now requires --select-from-stdin argument to work.
- Updated libraries

## Known issues

- EGUIs RetainedImage is deprecated. Will be removed in a future released of Aphorme

# 0.1.16

## Features

- Added option to retain focus on the launcher. Useful if starting multiple slow launching programs

## Known issues

- Focus retention only working with egui + X11. egui does not support focus grabbing on wayland

# 0.1.17

## Features

- Started work on Nix support

## Fixes

- Updated to egui 0.26 

# 0.1.18

## Features

- Nix support

## Fixes

- Removed erroneous error message on program launch

# 0.1.19

## Features

- Window size settable in 'gui_cfg'
- Font size settable in 'gui_cfg'

# 0.1.20

## Fixes

- Nix "Home Manager" path added to default paths. These should now be also searched.
