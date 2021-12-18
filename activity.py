"""
Author: Konnor Klercke
File: users.py
Purpose: Provide data on users to protobot.py
"""


#################
                #
import sqlite3  #                
                #
#################


#################################
                                #
USER_DATABSE = r'data/users.db' #
MAX_SCORE = 250                 #
                                #
#################################


def get_user_score(snowflake):
    cur.execute("""
    SELECT score 
    FROM users 
    WHERE snowflake = (?);
    """,
    (snowflake,))

    return cur.fetchone()[0]


def add_all_users(guild):
    for member in guild.members:
        add_user(member.id)


def add_user(snowflake):
    cur.execute("INSERT OR IGNORE INTO users VALUES (?, ?);", 
                 (snowflake, 0))
    conn.commit()


def change_user_score(snowflake, delta):
    cur.execute("""
    UPDATE users
    SET score = 
        ( CASE
            WHEN (score + ?) > ? THEN ?
            WHEN (score + ?) < 0 THEN 0
            ELSE score + ?
          END 
        )
    WHERE
        snowflake = ?
        """, 
        (delta, MAX_SCORE, MAX_SCORE, delta, delta, snowflake))
    conn.commit()


def init():
    global conn
    global cur
    conn = sqlite3.connect(USER_DATABSE)
    cur = conn.cursor()
    cur.execute("""CREATE TABLE IF NOT EXISTS users(
        snowflake INT PRIMARY KEY,
        score int);
        """)
    conn.commit()


def main():
    init()


if __name__ == "__main__":
    main()
