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
                                    #
#####################################


#####################################
                                    #
COMMAND_PREFIX = '!'                #
VERSION = "v0.1-alpha"              #
ACTIVITY = discord.Game("!help")    #
LOG_LEVEL = logging.INFO            #
                                    #
#####################################


# Load bot token from .env
load_dotenv()
TOKEN = os.getenv('DISCORD_TOKEN')

# Initialize bot object to use the COMMAND_PREFIX defined above
bot = commands.Bot(command_prefix=COMMAND_PREFIX)

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

    activity.readDatabase()

    for guild in bot.guilds:
        addAllUsersFromGuildToDatabase(guild)


@bot.event
async def on_disconnect():
    """
    Prints a message when bot disconnects from Discord. Usually this is temporary.
    """

    logging.warning('Lost connection to Discord.')

@bot.event
async def on_guild_join(guild):
    """
    Logs a message when bot joins a new guild.
    """

    logging.warning(f"Joined new guild: {guild.name + ' (' + str(guild.id) + ')'}")


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
        welcome_message = f"Welcome to the testing server, {member.name}!"

    else:
        welcome_message = f"Welcome to {member.guild.name}, {member.name}!"

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

    if message.author == bot.user:
        """
        Tells the bot to ignore its own messages
        """

        return 

    elif 'happy birthday' in message.content.lower():
        """
        Lets the bot say happy birthday whenever a user says it
        """

        mentions = message.mentions
        author = message.author.name + "(" + str(message.author.id) + ")"
        logging.info(f"{author} wished happy birthday to {len(mentions)} user(s).")
        for recipient in mentions:
            await message.channel.send(f"Happy Birthday <@{recipient.id}>! 🎈🎉🎂")

    elif 'im' in message.content.lower() or 'i\'m' in message.content.lower() or 'i am' in message.content.lower():
        """
        Lets the bot tell the famous "Hi x! I'm dad!" joke
        """

        user_message = message.content.split()
        
        ways_to_say_i_am = [' im', ' i\'m']

        for i in range(len(user_message)):
            if (' ' + user_message[i].lower() in ways_to_say_i_am):

                if len(user_message) - i < 2:
                    break 

                else:
                    response = "Hi " + " ".join(user_message[i + 1:]) + "! I'm dad!"
                    await message.channel.send(response)
                    break
    
    await bot.process_commands(message)


async def run_once_per_day():
    """
    Runs a block of code every day sometime between 00:00 and 01:00 local time.
    """

    await bot.wait_until_ready()

    if (int(time.strftime('%H', time.localtime())) < 1):
        # This code will run if it is the correct time
        logging.info("Running nightly operations.")
    else:
        logging.debug("Attempted to run daily event out of defined hours.")

    # Check every hour
    await asyncio.sleep(3600)


def addAllUsersFromGuildToDatabase(guild):
    for user in guild.members:
        if (not user.bot):
            addUserToDatabase(user.id, user.name)
    
    activity.writeDatabase()


def addUserToDatabase(uuid, name, score=0, allowmoderator=True, rankexempt=False):
    if (not uuid in activity.USERS.keys()):
                activity.addUser(uuid, name, score, allowmoderator, rankexempt)
                logging.info(f"Registered new user {name} ({uuid}) to database.")
    
    activity.writeDatabase()


bot.loop.create_task(run_once_per_day())
bot.run(TOKEN)
