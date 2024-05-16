#!/usr/bin/env python3

import git_stats
from Lindex import lindex


if __name__ == "__main__":
    # repos = git_stats.get_repos("some1and2-xc", git_stats.Source.Github)
    # print(repos)

    objects: lindex[bytes, bytes] = git_stats.raw_parsing.get_objects()
    head_object_key: str = git_stats.raw_parsing.get_head("main")

    head_object = objects[head_object_key]
    head_object_decoded = git_stats.raw_parsing.decompress_object(head_object)[1].decode("UTF-8")

    print(commit_value)

    # print(head_object_decoded)

    keys = ["tree", "parent"]

    for obj in head_object_decoded.split("\n"):
        for key in keys:
            if obj.startswith(key + " "):
                print(obj.split(" ")[0])
                sub_object = objects[bytes(obj.split(" ")[1], "utf-8")]
                v = git_stats.raw_parsing.decompress_object(sub_object)[1]
                try:
                    v = Commit.from_str(v.decode("utf-8"))
                except:
                    ...

                print(v, end="\n\n---\n\n")
            else:
                print(f"Object not in keys: '{obj}'")
