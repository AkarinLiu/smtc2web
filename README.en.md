# smtc2web

 A base of Rust's smtc2web achieve, Using in Live Stream Software show Playing Music.

**LinuxDO Reference Post:** https://linux.do/t/topic/937994

**[Join Translate](https://crowdin.com/project/smtc2web)** **[中文](./README.md)**

![OBS Connect to smtc2web Screenshot](./screenshot.png)

> The new icon combines Bootstrap's headphone image with Rust's Ferris character to create an icon of listening to music with headphones.

<img src="./src-tauri/icons/icon.png" alt="图标" width="64" height="64">

## Recommand IDE Setup 

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [TRAE](https://trae.com.cn/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Zed](https://zed.dev/)

## Linux MPRIS2 Support

smtc2web now supports reading currently playing media information on Linux via the **MPRIS2** (Media Player Remote Interfacing Specification) protocol. Most native Linux media players (VLC, Rhythmbox, Audacious, etc.) support MPRIS2 out of the box.

### Snap / Flatpak Sandbox Issues

Players installed via Snap or Flatpak (e.g. Firefox, Spotify) are sandboxed by default and **cannot be accessed by external programs over D-Bus**. You need to run the following commands to allow D-Bus access.

#### Snap

1. List installed Snap packages:
   ```bash
   snap list
   ```

2. Connect the `dbus` interface (using Firefox as an example):
   ```bash
   sudo snap connect firefox:dbus-daemon
   ```
   > If the `dbus-daemon` interface is not available, try:
   > ```bash
   > sudo snap connect firefox:session-dbus-observing
   > ```

3. Verify the interface is connected:
   ```bash
   snap connections firefox | grep dbus
   ```

4. **Restart the player** for changes to take effect.

#### Flatpak

1. Using Flatseal (GUI, recommended):
   ```bash
   flatpak install flathub com.github.tchx84.Flatseal
   ```
   Open Flatseal → select the target player → under **System Bus** or **Session Bus**, add `org.mpris.MediaPlayer2.*`.

2. Or override permissions via command line (using Firefox as an example):
   ```bash
   sudo flatpak override --socket=session-bus org.mozilla.firefox
   ```

3. **Restart the player** for changes to take effect.

### Troubleshooting MPRIS2

```bash
# Check for running MPRIS2 players
busctl --user list | grep mpris

# Manually query player status
gdbus call --session \
    --dest org.mpris.MediaPlayer2.<player-name> \
    --object-path /org/mpris/MediaPlayer2 \
    --method org.freedesktop.DBus.Properties.Get \
    org.mpris.MediaPlayer2.Player PlaybackStatus

# View smtc2web logs (located at ~/.local/share/smtc2web/logs/)
```

### Known Limitations

- **Snap Firefox**: Some Snap versions have very strict AppArmor policies — even connecting the `dbus` interface may not grant MPRIS2 access. In this case, install Firefox directly via your system package manager (`apt install firefox` or `dnf install firefox`).
- **Flatpak**: Some Flatpak apps do not expose the `org.mpris.MediaPlayer2.*` D-Bus interface; this requires explicit support from the app developer.

## Build

- [Windows](https://smtc2web.org/wiki/compile/windows)
