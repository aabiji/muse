Muse is a cli background music player.

Muse v2 checklist:
- Install script
- Bluetooth earbud pairing (pausing when disconnected?)
- Refactor the codebase (make sure to replace all unwraps with proper error propagation)

### Usage
| Command       | Description                      |
|---------------|----------------------------------|
| **muse play** | Play background music.           |
| **muse pause**| Pause background music.          |
| **muse start**| Start the playback server.       |
| **muse stop** | Stop the playback server.        |

### Installation
Install required dependencies:
```
# Arch based distro
sudo pacman -S alsa-lib

# Fedora based distro
sudo dnf install alsa-lib-devel

# Debian based distro
sudo apt install libasound2-dev
```

Install:
```
git clone https://github.com/aabiji/muse
cd muse
cargo build --release
sudo mv target/release/muse /usr/bin/muse
```

Uninstall:
```
rm ~/.muse.conf
sudo rm /usr/bin/muse
```

### Config
Muse can be configured. The config is stored
at `~/.muse.conf`. Here's all the possible options:
```
# A list of directory paths that
# contain audio files to be played. The paths
# need to be absolute. Muse will recursively
# search each directory path for audio files.
audio_directories = ["/path/to/audio_directory"]

# The order in which the audio files are played
# Can be either "Random" or "Alphabetical".
playback_order = "Random"

# Resume playback from where you left off?
resume_playback = true

# Number of seconds audio has been playing
# Used to resume playback. This is a value set
# by Muse so please don't edit this directly.
start_point = 0
```

Muse is granted under the MIT liscense. Contributions are welcome!