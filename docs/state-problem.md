## Problem

Zenkat should *asynchronously* load and index documents for best performance. However by doing so it becomes difficult to manage the main tree as there's no way to make tree manipulations memory safe if they happen asynchronously.

This causes substantial issues when dealing with client requests.

## Solution

Commands like `load_tree` should not directly manipulate the main tree. Instead they should return a working tree and an instruction on what to do to the main tree. This will be gathered by the main process and applied to the main tree, with an algorithm to resolve or flag conflicts.

This lets all changes be written with *copies* rather than *references*, therefore avoiding problems with the borrow checker.

## Other Solution

Don't run this code asynchronously and accept the (fairly large) performance hit for substantially simpler implementation.
