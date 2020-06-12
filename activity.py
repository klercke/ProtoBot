"""
Author: Konnor Klercke
File: users.py
Purpose: Provide data on users to protobot.py
"""


#####################################
                                    #
from dataclasses import dataclass   #
import csv                          #                
                                    #
#####################################


#################################
                                #
USER_DATABSE = 'data/users.csv' #
                                #
#################################


USERS = dict()


@dataclass
class User:

    uuid: int
    username: str
    score: int
    allowmoderator: bool
    rankexempt: bool

    def __str__(self):
        return f"{self.username}{{UUID: {self.uuid}, Score: {self.score}, AllowModerator: {self.allowmoderator}, RankExempt: {self.rankexempt}}}"

    
    def changeScore(self, delta):
        self.score += delta
        return self.score


def addUser(uuid, username, score=0, allowmoderator=True, rankexempt=False):
    newUser = User(uuid, username, score, allowmoderator, rankexempt)
    USERS[uuid] = newUser


def writeDatabase():
    with open('data/users.csv', mode='w', encoding='utf-8') as dataFile:
        fieldnames = ['uuid', 'username', 'score', 'allowmoderator', 'rankexempt']
        writer = csv.DictWriter(dataFile, fieldnames=fieldnames, lineterminator = '\n')

        writer.writeheader()
        for user in USERS.values():
            writer.writerow({
                'uuid': user.uuid, 
                'username': user.username, 
                'score': user.score, 
                'allowmoderator': user.allowmoderator, 
                'rankexempt': user.rankexempt})


def readDatabase():
    with open('data/users.csv', mode='r') as dataFile:
        dataFile = csv.DictReader(dataFile)
        for row in dataFile:
            user = User(int(row['uuid']), row['username'], int(row['score']), bool(row['allowmoderator']), (row['rankexempt']))
            USERS[int(row['uuid'])] = user


def printDatabase():
    for user in USERS.values():
            print(user)


def main():
    readDatabase()
    printDatabase()
    writeDatabase()


if __name__ == "__main__":
    main()
