# Swift bot

### Discord bot made with rust


- Using [serenity](https://github.com/serenity-rs/serenity)
- Tokio & other useful libs (see [cargo.toml](https://github.com/Okkonen-GitHub/Sbot/blob/main/Cargo.toml))
- Avatar from [vecteezy](https://www.vecteezy.com/free-vector/web)

### Running the bot

- Create and configure a .env file in the root directory as shown in the [.env.example](https://github.com/Okkonen-GitHub/Sbot/blob/main/.env.example)

#### development

Run by using `cargo run`

#### production

Run by using `cargo run --release`


### Music features, (this branch only)

**You MUST have Opus installed**

Arch: `sudo pacman -S opus`

Debian: `sudo apt install libopus-dev`

For the installation you will need some build tools

Arch: `sudo pacman -S base-devel`

Debian: `apt install build-essential autoconf automake libtool m4` 

It is recommended to install ffmpeg

For example on arch `sudo pacman -S ffmpeg` or on debian `sudo apt install ffmpeg`.

It is recommended to also install youtube-dl

With pip: `pip install youtube_dl`

Arch: `sudo pacman -S youtube-dl`

Debian: `sudo apt install youtube-dl`

For further insturctions have a look at this [repository](https://github.com/serenity-rs/songbird)