from enum import Enum
from requests import get
from Lindex import lindex
from collections import namedtuple
import pandas as pd


class Source(Enum):
    Selfhosted = 1
    Github = 2
    Gitlab = 3


Repos = namedtuple("repos", ["name", "description", "created_at", "updated_at", "git_url", "svn_url"])


def get_repos(user: str, source: Source) -> list[Repos]:
    """Function for getting the repos of a user based on site"""

    data = None

    match source:
        case Source.Github:
            res = get(f"https://api.github.com/users/{user}/repos").json()
            if len(res) == 0:
                data = []
            else:
                # Does magic to put all values into a Repos named tuple
                data = [Repos(
                    **{k: attr
                       for k, attr in repo.items()
                       if k in Repos._fields })
                for repo in res]
        case Source.Gitlab:
            ...
        case Source.Selfhosted:
            ...
        case _:
            raise ValueError("Invalid Source!")


    """
    # Important DF Columns
    These are the columns that should be included in every output
     - name
     - description
     - created_at
     - updated_at
     - git_url
     - svn_url
    """

    print(data)
    return data


if __name__ == "__main__":
    get_repos("some1and2-xc", Source.Github)
