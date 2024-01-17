Muse is a cli program to play background music.

### Usage
`muse`        Output program info.  
`muse play`   Play background music.  
`muse pause`  Pause background music.  
`muse stop`   Stops the background audio playback server.  

### Install
To install:
```
git clone https://github.com/aabiji/muse
cd muse
cargo build --release
sudo mv target/release/muse /usr/bin/muse
```

To uninstall:
```
rm ~/.muse.conf
sudo rm /usr/bin/muse
```

### Config
Muse can be configured. The config is stored
at `~/.muse.conf`. Here's all the possible options: 
```
# Path to the folder containing the audio files
# that will be played. The path needs to be absolute.
audio_folder_path = "/path/to/audio_folder"

# The order in which the audio files are played
# Can be either "Random" or "Alphabetical".
playback_order = "Random"

# Resume playback from where you left off?
resume_playback = true

# Number of seconds audio has been playing
# Used to resume playback. This is a value set
# by Muse so please don't edit this directly.
starting_point = 0
```

Muse is granted under the MIT liscense. Contributions are welcome!