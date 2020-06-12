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
                                    #
#####################################

#####################################
                                    #
COMMAND_PREFIX = '!'                #
VERSION = "v0.1-alpha"              #
ACTIVITY = discord.Game("!help")    #
                                    #
#####################################

load_dotenv()
TOKEN = os.getenv('DISCORD_TOKEN')

bot = commands.Bot(command_prefix=COMMAND_PREFIX)  


@bot.event
async def on_connect():
    """
    Prints a message with bot name and version when bot connects to Discord servers.
    Sets the bot activity to ACTIVITY.
    """

    print(f'{bot.user.name} {VERSION} has sucessfully connected to Discord.')
    await bot.change_presence(activity = ACTIVITY)


@bot.event 
async def on_ready():
    """
    Prints a list of guilds the bot is connected to when the bot is finished processing
    date from Discord servers.
    """

    print('Bot loading complete. Current guilds: ', end='')
    
    guilds = []
    for guild in bot.guilds:
        label = guild.name + " (" + str(guild.id) + ")"
        guilds.append(label)

    print(*guilds, sep=', ')


@bot.event
async def on_disconnect():
    """
    Prints a message when bot disconnects from Discord. Usually this is temporary.
    """

    print('Lost connection to Discord.')



@bot.event
async def on_member_join(member):
    """
    Direct-messages a user whenever the join the server
    """

    await member.create_dm()
    await member.dm_channel.send(
        f"Hi {member.name}, welcome to Konnor's Discord server. Please set your "
        "nickname to match the naming scheme used on the server. For example, if "
        "my name was John, my nickname would be \"Protobot | John\". Please also "
        "make sure to read any messages pinned in the #important channel."
    )


@bot.event
async def on_error(event, *args, **kwargs):
    """
    Writes to err.log whenever a message triggers an error
    """

    with open('err.log', 'a', encoding='utf-8') as errfile:
        if event == 'on_message':
            errfile.write(f'Unhandled message: {args[0]}\n')
        else:
            raise


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

        print("Wishing someone a happy birthday!")
        await message.channel.send('Happy Birthday! 🎈🎉🎂')

    await bot.process_commands(message)


    if 'im' in message.content.lower() or 'i\'m' in message.content.lower() or 'i am' in message.content.lower():
        """
        Lets the bot tell the famous "Hi x! I'm dad!" joke
        """

        user_message = message.content.split()
        
        ways_to_say_i_am = [' im', ' i\'m']

        for i in range(len(user_message)):
            if (' ' + user_message[i].lower() in ways_to_say_i_am):

                print("Dad joke incoming!")

                if len(user_message) - i < 2:
                    print("False alarm, user message too short!")
                    break 

                else:
                    response = "Hi " + " ".join(user_message[i + 1:]) + "! I'm dad!"
                    print(response)
                    await message.channel.send(response)
                    break
    

bot.run(TOKEN)