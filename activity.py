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

    
    def change_score(self, delta):
        if (self.score + delta > 0 and self.score + delta < 250):
            self.score += delta
        write_database()
        return self.score


def change_all_scores(delta):
    for user in USERS.values():
        if (user.score + delta > 0 and user.score + delta < 250):
            user.score += delta
    
    write_database()


def add_user(uuid, username, score=0, allowmoderator=True, rankexempt=False):
    newUser = User(uuid, username, score, allowmoderator, rankexempt)
    USERS[uuid] = newUser


def write_database():
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


def read_database():
    with open('data/users.csv', mode='r', encoding='utf-8') as dataFile:
        dataFile = csv.DictReader(dataFile)
        for row in dataFile:
            user = User(int(row['uuid']), row['username'], int(row['score']), bool(row['allowmoderator']), (row['rankexempt']))
            USERS[int(row['uuid'])] = user


def print_database():
    for user in USERS.values():
            print(user)


def main():
    read_database()
    print_database()
    write_database()


if __name__ == "__main__":
    main()
