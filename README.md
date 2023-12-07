<h1 align="center"><img width="500" src="kagu.png" /></h1>

# Kagu

Kagu aims to be a fast, private, self-hosted and lightweight alternative to a familiar chat service.

Kagu is powered by:
- 🚀 [rust] for stability, speed and security
- 🔊 [cpal] for audio playback and recording
- 👟 [quinn] to leverage the speed and reliability of QUIC
- 🖼️ [ratatui] for a rich terminal user interface

## Getting Started

Follow these instructions.

### Prerequisites

* To build on Debian or Ubuntu, install `libasound2-devel`.
* To build on Fedora install `alsa-lib-devel`.

### Clone the repo

```
git clone https://github.com/bblsh/kagu.git
```

### Running Kagu

To run the client, run:

```
cargo run --bin client -- -u username -a address -p port
```

This will establish a connection with the server  at `address:port` and display a TUI. Messages sent will be sent as `username`.

To run the server, run:

```
cargo run --bin server -- -a address -p port
```

The server will listen for QUIC connections on port `port`.

Without the `address` parameter, the server will listen on `0.0.0.0` by default.

## Navigating the Client Interface
To navigate through different panes (Messages, Channels, Input), use arrow keys.

To enter a text or voice channel, navigate to the Channels panel and press `Enter`.
- `Down` or `Up` will switch between text and voice channel sections.
- Press `Enter` to enter specific text or voice channels.
- While voice chat is live, press `Ctrl+D` to disconnect from a voice channel.
- `Esc` or `q` will exit selection and navigation of text or voice channels and place you in navigation mode.

To begin typing a message, press `i` and you will enter edit mode.

To send a message, press `Enter` while in edit mode.

To mention a user, type `@` and select a user by pressing `Tab` to autocomplete or `Enter` to send the message with that user mentioned.

Pressing `Esc` will exit edit mode.

`Ctrl+C` will disconnect the client and exit the program at any time.

`q` will disconnect and exit the program when not in edit mode.

### Realms and Channels
Realms and Channels can be added or removed.

To add a realm, navigate to the Realms pane, press `Enter`, then `Ctrl+a` to make a new realm. The input box to enter a realm code does nothing at the moment.

To remove a realm, navigate to the Realms pane, press `Enter`, then `Ctrl+r` when the realm to remove is highlighted. When prompted, press `Enter` to begin typing, and `Enter` again to confirm and remove.

Similarly, text and voice channels can be added by navigating to the Channels pane, pressing `Enter`, and `Ctrl+a` to add a channel. `Ctrl+r` will remove the channel.

`Esc` will exit focus from an input box, and pressing `q` will back out of a menu to add or remove a realm or channel.

## Notes / Known Issues

* Scrolling through chat history is not yet implemented (coming soonTM).
* There's currently no scrolling in text input (coming soonTM).
* Some features that are drawn out of bounds due to too small of a terminal size will panic the client.
* Kagu was used as motivation to learn Rust, so it is currently *very* unoptimized.
* There are many pieces of the code that are not consistent since new methods of doing things were learned as development progressed.
* Realms and channels are currently hardcoded as a proof of concept until a database is introduced to save and serve this data. Messages are not saved or persist because of this as well.
* Due to the current server design, audio is also echoed back to the user speaking.
* There's currently no option to choose an audio input and output (coming soonTM), so the defaults will be used.
* Any and all feedback or pull requests to improve Kagu in any way is welcome!
* This was tested on macOS, and `cpal` may have issues building or running on other platforms.
* Windows has not been fully tested yet.

[rust]: https://www.rust-lang.org/
[cpal]: https://github.com/RustAudio/cpal
[quinn]: https://github.com/quinn-rs/quinn
[ratatui]: https://github.com/tui-rs-revival/ratatui