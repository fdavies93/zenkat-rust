from argparse import ArgumentParser
from pathlib import Path
from dataclasses import dataclass
from enum import IntEnum
from types import prepare_class
from typing import Union
import re

class BlockType(IntEnum):
    DOCUMENT = 0
    PARAGRAPH = 1
    HEADING = 2
    H_RULE = 3
    LIST = 4
    LIST_ITEM = 7

class InlineType(IntEnum):
    BOLD = 5
    STR = 6

TokenType = Union[BlockType,InlineType]

@dataclass
class Token:
    type: TokenType
    content: list[Union["Token", str]]

TokenYield = tuple[list[Union[Token,str]], str]

def parse_paragraph(document: str) -> TokenYield:
    return ([document],"")

def parse_block_paragraph(document: str) -> TokenYield:
    regexp = re.compile(r"(([\S ]+)\n?)+")
    match = regexp.match(document)
    if not match: return ([],document)

    left = document[:match.end()]
    right = document[match.end()+1:]

    paragraph = parse_paragraph(left)
    
    token = Token(BlockType.PARAGRAPH, paragraph[0])     
    return ( [token], right )

def parse_block_list(document: str) -> TokenYield:
    regexp = re.compile(r"^[\t ]*(-|\*|\d\.).*")
    cur_document = document
    match = regexp.match(document)
    content = ""
    while match:
        content += cur_document[:match.end()]
        cur_document = cur_document[match.end()+1:]
        match = regexp.match(cur_document)

    # temporary while inline parsing not done
    token_list = []
    if len(content) > 0:
        token_list.append(Token(BlockType.LIST, [content]))
    
    return ( token_list , "")

# block := paragraph | heading | h_rule | list
def parse_blocks(document: str) -> list[Union[Token, str]]:
    cur_document = document
    tokens = []
    # we're consuming tokens
    while len(cur_document) > 0:
        for fn in [parse_block_list,parse_block_paragraph]:
            cur_yield = fn(cur_document)
            if len(cur_yield[0]) == 0: continue
            tokens.extend(cur_yield[0])
            # print(cur_yield)
            cur_document = cur_yield[1]
            break

    return tokens

def parse(document: str):
    return Token(BlockType.DOCUMENT, parse_blocks(document))

def main():
    parser = ArgumentParser()
    parser.add_argument("file")
    ns = parser.parse_args()

    with open(ns.file) as f:
        document = f.read()
    tree = parse(document)
    print(tree)
    
if __name__ == "__main__":
    main()
