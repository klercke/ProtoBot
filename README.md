# ProtoBot
A do-it-all Discord bot

This bot is a work in progress. It was originally made for my growing discord server, but is being expanded to be a general-purpose, modular bot. Any community contributions are welcome.

## Install
First install all the python modules needed:
```
sudo python3 -m pip install -U py-cord
sudo python3 -m pip install -U python-dotenv
sudo python3 -m pip install -U schedule
sudo python3 -m pip install -U sympy
```

After installing of of the modules, you will need to make a file in the ProtoBot folder called ".env" and paste the following into it:
```
# .env
DISCORD_TOKEN=[TOKEN]
```
where [TOKEN] is replaced with a [Discord Bot Token](https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token)

Note: To use the !tex command, you will need to make sure LaTeX is installed on the server running the bot (usually called TeXLive-full on Linux)