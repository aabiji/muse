Muse is a cli audio player. Just enter `muse play` and play away!

### Installation
```
# 1. Install the required dependencies:

# Arch based distro
sudo pacman -S alsa-lib

# Fedora based distro
sudo dnf install alsa-lib-devel

# Debian based distro
sudo apt install libasound2-dev

# 2. Install muse:
cargo install --git https://github.com/aabiji/muse
```

### Config
Muse can be configured. The config is stored
at `~/.muse.conf`. Here's all the possible options:
```toml
# A list of directory paths that
# contain audio files to be played. The paths
# need to be absolute. Muse will recursively
# search each directory path for audio files.
audio_directories = ["/path/to/audio_directory"]

# Should audio files be played at random?
randomize_tracks = true

# Resume playback from where you left off?
resume_playback = true

# Number of seconds audio has been playing
# Used to resume playback. This is a value set
# by Muse so please don't edit this directly.
start_point = 0
```

Muse is granted under the MIT liscense. Contributions are welcome!