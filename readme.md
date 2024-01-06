## muse
Muse is a cli program to play background music.

### Usage
`muse start` Starts playing background music.
`muse stop`  Stops playing background music.

### Config
See the [default config](config.toml).

### How it works
Muse is very simple. It uses a client-server model. The
client spawns the server if it has not already been spawned.
The client and server communicate via IPC. When the client
sends a REQUEST command, the server plays audio files (mp3, opus, etc)
from the specified audio folder in the specified order.
When the client sends a STOP command, the server pauses audio playback.
