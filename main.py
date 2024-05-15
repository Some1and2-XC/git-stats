#!/usr/bin/env python3

import git_stats

if __name__ == "__main__":
    # repos = git_stats.get_repos("some1and2-xc", git_stats.Source.Github)
    # print(repos)

    objects: bytes = git_stats.raw_parsing.get_objects()
