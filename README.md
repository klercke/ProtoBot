# ProtoBot

A Discord bot made for friends.
Free and open source, forever.

## Features

- **Dad jokes.**
    Tell ProtoBot you're hungry and it'll tell you it's dad.
    Every time on Father's Day.
- **Secret Santa.**
    Everyone signs up with their Steam profile, ProtoBot draws names and DMs each person their giftee with a link to their wishlist.
    Draw and gift dates show up as Discord events so nobody forgets.
- **Replies.**
    Wishes your friends a happy birthday when you do, and says you're welcome when you thank it.
- **Commands.**
    `/based`, `/age`, `/ping`, and a few others.
    Run `/help` to see them all.

## Usage

ProtoBot uses slash commands.
It also answers to `!` and "hey protobot":

```
/help
!help
hey protobot, help
```

### Secret Santa

An admin runs `/santa_create` and sets dates with `/santa_set_time`.
Everyone else joins with `/santa_register` and their Steam profile URL.
When it's time, an admin runs `/santa_shuffle` and ProtoBot sends out the DMs.

Make sure your DMs are open to server members or you won't get your giftee.

## Roadmap

See what's planned on the [roadmap](https://github.com/users/klercke/projects/1).

## Self-hosting

You'll need a Discord bot token and either a 64-bit Linux machine, [Nix](https://nixos.org/download/) with flakes enabled, or a container runtime (Docker, Podman, etc.).

1. Create an application in the [Discord Developer Portal](https://discord.com/developers/applications) and add a bot.
2. Under **Bot**, turn on the **Message Content Intent**.
    Without it, dad jokes and replies won't work.
3. For the binary or Nix, make a directory for ProtoBot to live in, with a `data` folder and a `.env` file containing your token:

   ```
   DISCORD_TOKEN=your-token-here
   ```

    The database is created at `data/protobot.sqlite3` and logs go to `logs/`.

    If running ProtoBot via container, you can skip this step.

4. Run it from that directory using one of the options below.

### Prebuilt binary

Download `protobot-x86_64-linux` and `protobot-x86_64-linux.sha256` from the [latest release](https://github.com/klercke/ProtoBot/releases/latest).
The binary is statically linked, so it should run on any modern 64-bit Linux distro without any dependencies.

```
sha256sum -c protobot-x86_64-linux.sha256
chmod +x protobot-x86_64-linux
./protobot-x86_64-linux
```

### Nix

```
nix run github:klercke/protobot
```

### Container

Images are published to the [GitHub Container Registry](https://github.com/klercke/ProtoBot/pkgs/container/protobot).

```
docker run -d --name protobot \
  -e DISCORD_TOKEN=your-token-here \
  -v protobot-data:/app/data \
  --restart unless-stopped \
  ghcr.io/klercke/protobot:latest
```

The database lives in the `protobot-data` volume, so it will survive an image update.
To update, pull the new image and recreate the container.
Podman will also work.

> [!TIP]
> When using the container image in a production environment, it is recommended to pin to a version tag instead of `latest`:
> `ghcr.io/klercke/protobot:0.11`

> [!IMPORTANT]
> If ProtoBot fails to start with `Failed to connect to SQLite database` and you're using the static binary or Nix release, make sure you created the `data/` directory in the same directory you're running ProtoBot from.

## Development

```
git clone https://github.com/klercke/protobot
cd protobot
nix develop
cargo run
```

Or `direnv allow` if you use direnv.

## Contributing

Bug reports, ideas, and PRs are welcome.
Please use the issue templates, and write commits using [Conventional Commits](https://www.conventionalcommits.org/).

## License

[GPL-3.0](LICENSE)
