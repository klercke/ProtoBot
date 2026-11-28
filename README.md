<h1>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/logo/lockup-dark.svg">
    <img src="assets/logo/lockup-light.svg" alt="ProtoBot" height="96">
  </picture>
</h1>

A Discord bot made for friends.
Free and open source forever.

[![REUSE status](https://api.reuse.software/badge/github.com/klercke/ProtoBot)](https://api.reuse.software/info/github.com/klercke/ProtoBot)
[![CI](https://github.com/klercke/ProtoBot/actions/workflows/ci.yml/badge.svg)](https://github.com/klercke/ProtoBot/actions/workflows/ci.yml)

## Features

- **Dad jokes.**
    Tell ProtoBot you're hungry and it'll tell you it's dad.
    (Every time on Father's Day.)
- **Secret Santa.**
    Everyone signs up with their Steam profile, and ProtoBot draws names at the draw time and DMs each person their giftee with a link to their wishlist.
    Reminders go out before gifts are due and anyone who forgets can ask ProtoBot who they're shopping for.
    Draw and gift dates show up as Discord events and participants can get a role so they're easy to find.
- **Replies.**
    Wishes your friends a happy birthday when you do, and says you're welcome when you thank it.
- **Commands.**
    `/based`, `/age`, `/ping`, and a few others.
    Run `/help` to see them all.

## Usage

ProtoBot uses slash commands.

```
/help
```

### Secret Santa

An admin runs `/santa_create` with the draw and gift times, plus an optional budget and rules link.
Times can be changed later with `/santa_set_time`, and the budget and rules with `/santa_set_details`.
To give everyone who signs up a role, use `/santa_set_role` (and `/santa_clear_role` to stop).

Everyone else joins with `/santa_register` and their Steam profile URL, and can check the details with `/santa_info`.
At the draw time, ProtoBot draws names and DMs everyone their giftee.
An admin can also draw early with `/santa_shuffle`.
ProtoBot sends a reminder 24 hours before gifts are due, which admins can change with the `remind_hours` option on `/santa_set_time`.
Forgot who you're shopping for? `/santa_my_giftee` will tell you, and only you can see the answer.

Make sure your DMs are open to server members or you won't get your giftee.

## Roadmap

See what's planned on the [roadmap](https://github.com/users/klercke/projects/1).

## Preview vs. Production

There are currently two versions of ProtoBot that are publicly available: **ProtoBot Preview** and **ProtoBot Production** (or as I like to call it, ProdoBot).
**ProtoBot Production** is usually just called **ProtoBot**.

**ProtoBot Preview** is built from `main` on every change.
The ProtoBot project uses fairly robust CI, lints, checks, and automated tests, plus manual testing where necessary, so these builds should be free of any catastrophic bugs, but they should **NOT** be considered stable.

**ProtoBot Production** runs the [latest release](https://github.com/klercke/ProtoBot/releases/latest) build.
Changes are tested in the real world (via ProtoBot Preview) before being released, so ProtoBot Production is ideally more stable.

The deployments for both versions of ProtoBot are done via GitHub Actions and are [publicly auditable](https://github.com/klercke/ProtoBot/actions).

Common to any instance of ProtoBot is the fact that this is **pre-1.0 code** from a **novice** Rust programmer.
ProtoBot is my longest-running project and I take a great deal of care to make it as stable as possible (I use Preview every day in a server with over 100 people!), but there will be bugs from time to time.
If you do discover any bugs or missing functionality, please report them via a [GitHub issue](https://github.com/klercke/ProtoBot/issues/new/choose).

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
> `ghcr.io/klercke/protobot:X.Y`

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

All code in this repository is licensed under [AGPL-3.0-or-later](./LICENSES/AGPL-3.0-or-later.txt), unless otherwise stated.
This repository's GitHub workflows and CI tooling configurations are licensed under [CC0-1.0](./LICENSES/CC0-1.0.txt).
The ProtoBot logo(s) and related assets are licensed under [CC-BY-SA-4.0](./LICENSES/CC-BY-SA-4.0.txt).
The lockups combine the logo ([CC-BY-SA-4.0](./LICENSES/CC-BY-SA-4.0.txt)) with wordmark outlines from the Audiowide font, licensed under [OFL-1.1](./LICENSES/OFL-1.1.txt).

See [REUSE.toml](./REUSE.toml) for exactly which files are under each license.
