# This is a mock implementation of the server for Zenkat
# in Python, designed to test routing schemes and algorithms
# without the overhead of writing correct Rust.

from flask import Flask
from dataclasses import dataclass
from enum import IntEnum

# interesting problem: we don't know which files exist until we crawl the
# directory structure. however crawling that structure is what constitutes
# a 'load'

# loading state might be too complex to put in the tree metadata
# instead, perhaps this needs to query the root node of the tree

# also there will need to be a specific technical definition of each state
# for folders, since it's non-obvious what a partial load vs a full load
# actually means - we can load 'partially' as in only a subset of files
# or 'partially' as in only metadata about a file
# i.e. this could be 'horizontal' or 'vertical'

class TreeState(IntEnum):
    UNLOADED = 0
    LOADING = 1
    LOADED_METADATA = 2
    LOADED_FULL = 3

@dataclass
class Tree:
    name: str
    path: str # this should probably be a virtual path

app = Flask(__name__)

@app.route("/tree", methods=['GET'])
def get_trees():
    # get all possible trees
    # i.e. subfolders of folders designated in config
    # return a list of their names
    return ""

@app.route("/tree", methods=['PUT'])
def put_tree():
    return ""

@app.route("/tree/<name>", methods=['GET'])
def get_tree():
    # if not tree_is_loaded(name): load_tree(name)
    # return tree
    return ""
