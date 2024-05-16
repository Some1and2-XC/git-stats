import os
import zlib
import base64
from glob import glob

from Lindex import lindex

def get_objects() -> lindex[bytes, bytes]:
    """Gets git objects from ./.git/objects/**"""

    objects = lindex(debug=False)
    for filename in glob("./.git/objects/*/*"):
        with open(filename, "rb") as f:
            key = "".join(filename.split(os.path.sep)[-2::]).encode("UTF-8")
            objects[key] = f.read()

    return objects


def get_head(branch: str) -> bytes:
    """Gets the starting object by commit name"""

    with open(
        os.path.sep.join(
            f"./.git/refs/heads/{branch}".split("/")
            ), "rb") as f:
        branch_data = f.read().strip()

    return branch_data


def decompress_object(data: bytes) -> bytes:
    """Function for decompressing git objects"""
    return zlib \
        .decompress(data) \
        .split(b"\x00")  # Splits on null character
        # .replace(b"\x00", b"\n")  # Replaces Null Character with new-line


if __name__ == "__main__":
    objects = get_objects()
    branch = get_head("main")

    if branch in objects:
        data = objects[branch]
    else:
        print(f"Can't find object '{branch}'")
        exit(1)

    print(v)
    print(v.decode("UTF-8"))
