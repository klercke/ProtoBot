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
from discord.ext import tasks       #
import logging                      #
import time                         #
import asyncio                      #
import activity                     #
import string                       #
import schedule                     #
import threading                    #
                                    #
#####################################


#####################################
                                    #
COMMAND_PREFIX = '!'                #
VERSION = "v0.5.1-alpha"            #
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

    logging.warning(f'{bot.user.name} {VERSION} has successfully connected to Discord.')
    await bot.change_presence(activity = ACTIVITY)


@bot.event
async def on_ready():
    """
    Prints a list of guilds the bot is connected to when the bot is finished processing
    date from Discord servers. Also calls the initializing functions in activity.py.
    """

    logging.info('Bot loading complete. Current guilds: ')

    for guild in bot.guilds:
        label = guild.name + " (" + str(guild.id) + ")"
        logging.info(label)

    activity.read_database()

    for guild in bot.guilds:
        add_all_users_from_guild_to_database(guild)

    # Start scheduled events
    logging.info("Scheduling events...")
    logging.info("Daily events")
    schedule.every().day.at("00:00").do(run_once_every_day)
    logging.info("Hourly events")
    schedule.every().hour.do(run_once_every_hour)
    logging.info("Minutely events")
    schedule.every().minute.do(run_once_every_minute)
    logging.info("Scheduling complete!")
    t = threading.Thread(target=schedule_worker)
    t.start()


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
            f"Hi {member.name}, welcome to Lounge server. Please set your "
            "nickname to match the naming scheme used on the server. For example, if "
            "my name was John, my nickname would be \"ProtoBot | John\". Please also "
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

    if not message.content:
        """
        Tells the bot to ignore messages with no text content
        """

        return

    if message.channel.id == 759892648827551745 or message.channel.id == 759896557578354718:
        """
        Tells the bot not to respond in #introduce yourself of mod-lounge
        """

        return

    if message.channel.name.lower() == 'counting':
        # Check to see if message is an int
        invalid_message = False
        try:
            int(message.content)
        except ValueError:
            invalid_message = True

        if not invalid_message:
            # Check to make sure the message is the next int sequentially
            channel_history = await message.channel.history(limit=2).flatten()
            if len(channel_history) != 1:
                last_msg = channel_history[1]
                if (int(message.content) != int(last_msg.content) + 1):
                    await message.channel.send(f"<@{message.author.id}> sent an incorrect number! Counting is now over!")
                    await message.channel.set_permissions(message.guild.default_role, send_messages = False)
        else:
            await message.channel.send(f"<@{message.author.id}> sent an incorrect number! Counting is now over!")
            await message.channel.set_permissions(message.guild.default_role, send_messages = False)

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

    elif 'im' in message.content.lower() or 'i\'m' in message.content.lower() or 'i`m' in message.content.lower() or \
        'i‘m' in message.content.lower() or 'i´m' in message.content.lower() or 'i am' in message.content.lower() or \
        'i’m' in message.content.lower():
        """
        Lets the bot tell the famous "Hi x! I'm dad!" joke
        """

        user_message = message.content.split()

        ways_to_say_i_am = [' im', ' i\'m', ' i´m', ' i`m', ' i‘m', ' i’m']

        for i in range(len(user_message)):
            if ' ' + user_message[i].lower() in ways_to_say_i_am:

                if len(user_message) - i < 2:
                    break

                else:
                    response = "Hi " + " ".join(user_message[i + 1:]) + "! I'm dad!"
                    await message.channel.send(response)
                    break

            elif user_message[i].lower() == "i" and len(user_message) >= i:

                if user_message[i + 1].lower() == "am" and len(user_message > 2):

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


@bot.command(name="strange", help="Sends strange.png")
async def strange(ctx):
    await ctx.message.channel.send(file=discord.File('resources/strange.png'))


@bot.command(name="based", help="Uses a complex algorithm to determine whether or not a user is based.")
async def based(ctx):
    ways_to_say_based = [
        "Based.",
        "Based and redpilled.",
        "Based and Teddypilled.",
        "Based? Based on what?",
        "Based upon pillars of salt and pillars of sand.",
        "Not based.",
        "Cringe and bluepilled.",
        "CEO of the Based Department.",
        "Enemy of the based.",
        "All of your based are belong to us."
    ]

    response = random.choice(ways_to_say_based)

    if (ctx.message.mentions != []):
        uuid = ctx.message.mentions[0].id
        response = f"<@{uuid}> is " + response

    await ctx.send(response)


@bot.command(name="about", help="Displays some information about ProtoBot.")
async def about(ctx):
    await ctx.message.channel.send(f"ProtoBot {VERSION}. Source code and bug tracker: https://github.com/klercke/ProtoBot")


@bot.command(name="poll", help="Creates a poll. Usage: !poll \"QUESTION\" TIME[S/M/H/D] EMOJI1 EMOJI2 ...")
async def poll(ctx):
    """
    Allows users to create timed polls
    """

    user_input = ctx.message.content
    user = ctx.message.author
    
    try:
        # Get the user's question
        prompt = ""
        writing = False
        count = 0
        for character in user_input:
            count += 1
            if not writing and character == "\"":
                writing = True
            elif writing:
                if character == "\"":
                    writing = False
                    break
                else:
                    prompt += character
            elif count == len(user_input) - 1:
                await ctx.channel.send("Sorry, I couldn't understand your command. Please make sure the poll question is in quotes.")
                return

        # Get the TTL for the poll
        user_input = user_input[count + 1:]
        if not user_input:
            await ctx.channel.send("You didn't provide a time. Assuming 5 minutes.")
            poll_time = "5m"
        else:
            poll_time = ""
            count = 0
            for character in user_input:
                count += 1
                if character != " ":
                    poll_time += character
                else:
                    break

        unit_long = ""
        unit = poll_time[-1].lower()
        poll_time = int(poll_time[:-1])
        poll_time_in_sec = 0
        if unit == 's':
            unit_long = "seconds"
            poll_time_in_sec = poll_time
        elif unit == 'm':
            unit_long = "minutes"
            poll_time_in_sec = poll_time * 60
        elif unit == 'h':
            unit_long = "hours"
            poll_time_in_sec = poll_time * 3600
        elif unit == 'd':
            unit_long = "days"
            poll_time_in_sec = poll_time * 86400


        # Get emoji options
        user_input = user_input[count:]
        user_input = user_input.split()
        options = []
        message_sent = await ctx.message.channel.send(f"<@{user.id}> has started a poll:\n{prompt}\nVoting will last {poll_time} {unit_long}.")
        for emoji in user_input:
            options += emoji
            await message_sent.add_reaction(emoji)
    except:
        await ctx.message.channel.send(f"<@{user.id}>, something went wrong with your command. Please make sure to use proper syntax:\n!poll \"QUESTION\" TIME[S/M/H/D] EMOJI1 EMOJI2 ...")
        return

    async def count_poll_results(message_sent, poll_time_in_sec):
        # Wait for voting to finish
        await asyncio.sleep(poll_time_in_sec)

        # Create dictionary for results
        results = {}
        for option in options:
            results[option] = 0

        # Get original message
        message_sent = await message_sent.channel.fetch_message(message_sent.id)

        # Count results 
        total_votes = 0
        for reaction in message_sent.reactions:
            results[reaction.emoji] = reaction.count - 1
            total_votes += reaction.count - 1

        if total_votes == 0:
            await message_sent.channel.send(f"Voting for \"{prompt}\" complete. Nobody voted!")
        else:
            # Find winner
            winner = max(results, key = results.get)

            # Check for ties
            tie = False
            for result in results.keys():
                if results[result] == results[winner]:
                    tie = True

            # If there's a tie, find out which results tied and make a string out of them, then print results
            if tie:
                tied_emojis = []
                tie_string = ""
                for result in results.keys():
                    if results[result] == results[winner]:
                        tied_emojis.append(result)
                num_ties = len(tied_emojis)
                for i in range(num_ties - 1):
                    tie_string += f"{tied_emojis[i]}, "
                tie_string += f"and {tied_emojis[i + 1]}"
                # Remove comma for only 2 results
                if num_ties == 2:
                    tie_string = tie_string.replace(',', '')

                await message_sent.channel.send(f"Voting for \"{prompt}\" complete. There was a {num_ties}-way tie between {tie_string} with {results[winner]} ({winner_percentage}%) votes each.")

            # Otherwise, show winner
            else:
                winner_percentage = round((results[winner] / total_votes) * 100, 2)
                await message_sent.channel.send(f"Voting for \"{prompt}\" complete. {winner} is the winner with {results[winner]} ({winner_percentage}%) votes!")

    await count_poll_results(message_sent, poll_time_in_sec)
    

def run_once_every_day():
    """
    Runs a block of code every day.
    """

    pass



def run_once_every_minute():
    """
    Runs a block of code every minute.
    """

    # Give every user in a voice channel points
    for guild in bot.guilds:
        for channel in guild.voice_channels:
            for user in channel.members:
                change_user_score(user.id, POINTS_PER_MINUTE_TALKING)


def run_once_every_hour():
    """
    Runs a block of code every hour.
    """

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


def schedule_worker():
    logging.info("Schedule worker thread initialized.")
    while True:
        schedule.run_pending()
        time.sleep(1)


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

    bot.run(TOKEN)

if __name__ == "__main__":
    main()
