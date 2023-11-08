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
