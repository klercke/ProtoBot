"""
Author: Konnor Klercke
File: protobot.py
Purpose: Discord bot for my personal server
"""


#################################
                                #
import os                       #
import discord                  #
from dotenv import load_dotenv  #
import random                   #
                                #
#################################


load_dotenv()   # Loads .env file
TOKEN = os.getenv('DISCORD_TOKEN')  # Tells the program to use the token provided in .env  
client = discord.Client()   # Tells the program to use the default Client class


@client.event   # Logs a sucessful connection to the Discord server
async def on_ready():
    print(f'{client.user.name} has sucessfully connected to the server!')


@client.event   # Responds to a new member joining the server
async def on_member_join(member):
    await member.create_dm()    # Creates a DM with the user and sends a message
    await member.dm_channel.send(
        f"Hi {member.name}, welcome to Konnor's Discord server. Please set your "
        "nickname to match the naming scheme used on the server. For example, if "
        "my name was John, my nickname would be \"Protobot | John\". Please also "
        "make sure to read any messages pinned in the #important channel."
    )


@client.event   # Responds to a user sending a message in the server
async def on_message(message):
    if message.author == client.user:
        return  # Ensures ProtoBot will not respond to its own messages
    
    b99_quotes = [
        "I'm the human form of the 💯 emoji.",
        "Bingpot!",
        "Nine-nine!",
        (
            "Cool. Cool cool cool cool cool cool cool, "
            "no doubt no doubt no doubt no doubt."
        ),
        "With all due respect, I am gonna completely ignore everything you just said.",
        (
            "The English language can not fully capture the depth and complexity of my "
            "thoughts, so I’m incorporating emojis into my speech to better express myself. 😉."
        ),
        "I’d like your $8-est bottle of wine, please.",
        "But if you’re here, who’s guarding Hades?"
    ]

    ways_to_say_i_am = ["im", "i'm", "i am"]

    if message.content == '99!':
        response = random.choice(b99_quotes)
        await message.channel.send(response)    # Sends a Brooklyn 99 quote whenever a user says '99!'

    elif 'i\'m ' or 'im ' or 'i am ' in message.content.lower():
        user_message = message.content.split()
        for i in range(len(user_message)):
            if len(user_message) - i < 2:
                break
            
            elif user_message[i].lower() in ways_to_say_i_am:
                await message.channel.send("Hi " + " ".join(user_message[i + 1:]) + "! I'm dad!")
                break

            elif len(user_message) - i < 3:
                break
            
            elif user_message[i].lower() + " " + user_message[i + 1].lower() in ways_to_say_i_am:
                await message.channel.send("Hi " + " ".join(user_message[i + 2:]) + "! I'm dad!")
                break
    


client.run(TOKEN)