from enum import Enum
from requests import get
from Lindex import lindex


class Source(Enum):
    Selfhosted = 1
    Github = 2
    Gitlab = 3

def get_repos(user: str, source: Source):
    """Function for getting the repos of a user based on site"""

    match source:
        case Source.Github:
            res = get(f"https://api.github.com/users/{user}/repos")
        case _:
            raise ValueError("Invalid Source!")

    v = res.json()
    v = lindex(v)

    v.pprint()

if __name__ == "__main__":
    get_repos("some1and2-xc", Source.Github)
