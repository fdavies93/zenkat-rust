# This is a mock implementation of the server for Zenkat
# in Python, designed to test routing schemes and algorithms
# without the overhead of writing correct Rust.

from flask import Flask
from dataclasses import dataclass
from enum import IntEnum, auto

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

# it doesn't matter on modern hardware: even 100,000 files in a single tree is only a few MB of RAM to store *metadata*
# implementing partial metadata loading probably a waste of time
# implementing partial *file* loading probably not

# probably the hydration / dehydration terminology is useful here for *file* states

class TreeState(IntEnum):
    UNLOADED = auto() # only the name of the tree is known
    LOADING_METADATA = auto() # transitional state
    LOADED_METADATA_ONLY = auto() # all metadata, no files hydrated
    LOADED_PARTIAL = auto() # all metadata, some files hydrated
    LOADED_FULL = auto() # all metadata, all files hydrated    

# since almost every MD file is going to be less than 1MB
# it doesn't make much sense to get more granular than this
class FileState(IntEnum):
    DEHYDRATED = auto()
    HYDRATING = auto()
    HYDRATED = auto()

    
@dataclass
class Tree:
    name: str
    path: str # this should probably be a virtual path
    state: TreeState
    root_node: str | None # id of root node

@dataclass
class Node:
    id: str
    path: str # virtual path to node

app = Flask(__name__)

@app.route("/tree", methods=['GET'])
def get_trees():
    # get all possible trees
    # i.e. subfolders of folders designated in config
    # return a list of their names
    return ""


@app.route("/tree", methods=['PUT'])
def put_tree():
    # normally should be used for changing the tree state
    return ""


@app.route("/tree/<name>", methods=['GET'])
def get_tree():
    # if not tree_is_loaded(name): load_tree(name)
    # return tree
    return ""


@app.route("/tree/<name>/query", methods=["POST"])
def post_tree_query():
    # query the tree
    # could return different data types depending on
    # query
    return ""


@app.route("/tree/<name>/node", methods=["GET"])
def list_nodes():
    # list all the nodes that are loaded in this tree
    # with their IDs or some other abbreviated form
    # could return error if tree not loaded, or just
    # automagically load it
    return ""


@app.route("/tree/<name>/node/<node_id>", methods=["GET"])
def get_node():
    # get one node by id with full detail
    return ""
