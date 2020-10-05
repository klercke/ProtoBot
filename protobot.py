"""
Author: Konnor Klercke
File: protobot.py
Purpose: Discord bot for my personal server
"""


#####################################
                                    #
import os                           #
import discord                      #
from dotenv import load_dotenv      #
import random                       #
from discord.ext import commands    #
import logging                      #
import time                         #
import asyncio                      #
import activity                     #
import string                       #
                                    #
#####################################


#####################################
                                    #
COMMAND_PREFIX = '!'                #
VERSION = "v0.3.4-alpha"            #
ACTIVITY = discord.Game("!help")    #
LOG_LEVEL = logging.INFO            #
                                    #
#####################################


#################################
                                #
POINTS_PER_MESSAGE = 2          #
POINTS_PER_MINUTE_TALKING = 1   #
POINT_DECAY_PER_HOUR = 1        #
                                #
#################################


# Initialize bot object to use the COMMAND_PREFIX defined above
bot = commands.Bot(command_prefix=COMMAND_PREFIX)


@bot.event
async def on_connect():
    """
    Prints a message with bot name and version when bot connects to Discord servers.
    Sets the bot activity to ACTIVITY.
    """

    logging.warning(f'{bot.user.name} {VERSION} has sucessfully connected to Discord.')
    await bot.change_presence(activity = ACTIVITY)


@bot.event 
async def on_ready():
    """
    Prints a list of guilds the bot is connected to when the bot is finished processing
    date from Discord servers. Also calls the initizaling functions in activity.py.
    """

    logging.info('Bot loading complete. Current guilds: ')
    
    for guild in bot.guilds:
        label = guild.name + " (" + str(guild.id) + ")"
        logging.info(label)

    activity.read_database()

    for guild in bot.guilds:
        add_all_users_from_guild_to_database(guild)


@bot.event
async def on_disconnect():
    """
    Prints a message when bot disconnects from Discord. Usually this is temporary.
    """

    logging.warning('Lost connection to Discord.')


@bot.event
async def on_guild_join(guild):
    """
    Logs a message when bot joins a new guild and adds all users from that guild to the database.
    """

    logging.warning(f"Joined new guild: {guild.name + ' (' + str(guild.id) + ')'}")

    add_all_users_from_guild_to_database(guild)


@bot.event
async def on_member_join(member):
    """
    Direct-messages a user whenever the join a server
    """

    await member.create_dm()
    if (member.guild.id == 150717946333888514):
        welcome_message = (
            f"Hi {member.name}, welcome to Konnor's Discord server. Please set your " 
            "nickname to match the naming scheme used on the server. For example, if "
            "my name was John, my nickname would be \"Protobot | John\". Please also "
            "make sure to read any messages pinned in the #important channel."
        )

    elif (member.guild.id == 720996920939642912):
        welcome_message = f"Welcome to the ProtoBot development server, {member.name}!"

    else:
        welcome_message = f"Welcome to {member.guild.name}, {member.name}!"

    add_user_to_database(member.id, member.name)

    await member.dm_channel.send(welcome_message)


@bot.event
async def on_error(event, *args, **kwargs):
    """
    Writes to err.log whenever a message triggers an error
    """

    if event == 'on_message':
        logging.error(f'Unhandled message: {args[0]}')
    else:
        logging.exception(event)


@bot.event
async def on_message(message):
    """
    Allows the bot to respond to user messages rather than commands
    """

    if message.author.bot:
        """
        Tells the bot to ignore its own messages
        """

        return

    if message.channel.id = 759892648827551745 or message.channel.id = 759896557578354718:
        """
        Tells the bot not to respond in #introduce yourself of mod-lounge
        """

        return

    if message.content[0] != "!":
        change_user_score(message.author.id, POINTS_PER_MESSAGE)

    if 'happy birthday' in message.content.lower():
        """
        Lets the bot say happy birthday whenever a user says it
        """

        mentions = message.mentions
        author = message.author.name + "(" + str(message.author.id) + ")"
        logging.info(f"{author} wished happy birthday to {len(mentions)} user(s).")
        for recipient in mentions:
            await message.channel.send(f"Happy Birthday <@{recipient.id}>! 🎈🎉🎂")

    #elif 'er' in message.content.lower():
    #   """
    #   Lets the bot tell the famous "x-er? I hardly know 'er!" joke
    #   """

    #   user_message = message.content.split()

    #   for i in range(len(user_message)):
    #       user_message[i] = user_message[i].translate(str.maketrans('', '', string.punctuation))
    #       if (user_message[i][-2:] == "er" and len(user_message[i]) > 4):
    #           response = user_message[i][0].upper() + user_message[i][1:] + "? I hardly know her!"
    #           await message.channel.send(response)
    #           break

    elif 'im' in message.content.lower() or 'i\'m' in message.content.lower() or 'i`m' in message.content.lower() or \
        'i‘m' in message.content.lower() or 'i´m' in message.content.lower() or 'i am' in message.content.lower():
        """
        Lets the bot tell the famous "Hi x! I'm dad!" joke
        """

        user_message = message.content.split()
        
        ways_to_say_i_am = [' im', ' i\'m', ' i´m', ' i`m', ' i‘m']

        for i in range(len(user_message)):
            if ' ' + user_message[i].lower() in ways_to_say_i_am:

                if len(user_message) - i < 2:
                    break 

                else:
                    response = "Hi " + " ".join(user_message[i + 1:]) + "! I'm dad!"
                    await message.channel.send(response)
                    break

            elif user_message[i].lower() == "i" and len(user_message) >= i:

                if user_message[i + 1].lower() == "am":

                    response = "Hi " + " ".join(user_message[i + 2:]) + "! I'm dad!"
                    await message.channel.send(response)
                    break
    
    await bot.process_commands(message)


@bot.command(name="score", help="Displays your current score, or someone else's score if mentioned")
async def check_user_score(ctx):
    if (ctx.message.mentions == []):
        uuid = ctx.message.author.id
        score = get_user_score(uuid)
        await ctx.message.channel.send(f"Score for <@{uuid}>: {score}")
    else:
        for user in ctx.message.mentions:
            uuid = user.id
            score = get_user_score(uuid)
            await ctx.message.channel.send(f"Score for <@{uuid}>: {score}")

    


@bot.command(name="correct", help="Sends correct.png")
async def correct(ctx):
    await ctx.message.channel.send(file=discord.File('resources/correct.png'))


@bot.command(name="what", help="Sends what.png")
async def what(ctx):
    await ctx.message.channel.send(file=discord.File('resources/what.png'))


async def run_once_every_day():
    """
    Runs a block of code every day sometime between 00:00 and 01:00 local time.
    """

    if (int(time.strftime('%H', time.localtime())) < 1):
        # This code will run if it is the correct time
        logging.info("Running nightly operations.")
    else:
        logging.debug("Attempted to run daily event out of defined hours.")


async def run_once_every_minute():
    """
    Runs a block of code every minute
    """

    await asyncio.sleep(60)

    # Give every user in a voice channel points
    for guild in bot.guilds:
        for channel in guild.voice_channels:
            for user in channel.members:
                change_user_score(user.id, POINTS_PER_MINUTE_TALKING)


async def run_once_every_hour():
    """
    Runs a block of code every hour
    """

    await asyncio.sleep(3600)

    # Call the once-each-day function so it can do its check
    await run_once_every_day()

    activity.change_all_scores(-POINT_DECAY_PER_HOUR)


def change_user_score(uuid, delta):
    if (uuid in activity.USERS.keys()):
        activity.USERS[uuid].change_score(delta)
    else:
        logging.error(f"Attempted to change score of user {uuid} when user is not in database.")


def get_user_score(uuid):
    if (uuid in activity.USERS.keys()):
        return activity.USERS[uuid].score
    else:
        logging.error(f"Attempted to get score of user {uuid} when user is not in database.")


def add_all_users_from_guild_to_database(guild):
    for user in guild.members:
        if (not user.bot):
            add_user_to_database(user.id, user.name)
    
    activity.write_database()


def add_user_to_database(uuid, name, score=0, allowmoderator=True, rankexempt=False):
    if (not uuid in activity.USERS.keys()):
                activity.add_user(uuid, name, score, allowmoderator, rankexempt)
                logging.info(f"Registered new user {name} ({uuid}) to database.")
    
    activity.write_database()




def main():
    # Load bot token from .env
    load_dotenv()
    TOKEN = os.getenv('DISCORD_TOKEN')

    # Generate timestamp of startup
    timestamp = time.strftime('%Y%m%d-%H%M%S')

    # Configure logging
    logging.basicConfig(
        level = LOG_LEVEL, 
        format = '%(asctime)s: [%(levelname)s] - %(message)s',
        datefmt = '%Y-%m-%d %H:%M:%S', 
        handlers = [
            logging.FileHandler(f"logs/{timestamp}.log", mode = "w"), 
            logging.StreamHandler()
        ]
    )

    bot.loop.create_task(run_once_every_minute())
    bot.loop.create_task(run_once_every_hour())

    bot.run(TOKEN)

if __name__ == "__main__":
    main()
