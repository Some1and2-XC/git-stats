#!/usr/bin/env python3

import git_stats

if __name__ == "__main__":
    git_stats.get_repos("some1and-xc", git_stats.Source.Github)
