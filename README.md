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

You'll need [Nix](https://nixos.org/download/) with flakes enabled and a Discord bot token.

1. Create an application in the [Discord Developer Portal](https://discord.com/developers/applications) and add a bot.
2. Under **Bot**, turn on the **Message Content Intent**.
    Without it, dad jokes and replies won't work.
3. Make a directory for ProtoBot to live in, with a `data` folder and a `.env` file containing your token:

   ```
   DISCORD_TOKEN=your-token-here
   ```

4. Run it from that directory:

   ```
   nix run github:klercke/protobot
   ```

The database is created at `data/protobot.sqlite3` and logs go to `logs/`.

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
