# Sokoban

Idk what you wanna hear from me, this is sokoban but in rust.

Just use cargo to build it and run it. It'll run. Probably.

Play any of David W. Skinner's sokoban puzzle sets (and others if you format them the same as the included .txt) by specifying the file path to the .txt file when launching the program.\
Example:
```bash
./sokoban --file ~/puzzleset.txt
```

### Symbols and their meaning:
 - #: Wall
 - o: cube
 - O: cube on button
 - x: button
 - K: player
 - F: finish (optionally implementable into levels, only active when all buttons are pressed)

### Credits
microban.txt created by David W. Skinner\
http://abelmartin.com/rj/sokobanJS/Skinner/David%20W.%20Skinner%20-%20Sokoban.htm

### AI Use Disclosure
Due to the time i was given for this project and the fact that i didn't know rust beforehand, there were times i didn't know WHY things weren't working. AI was used to put me back on the right track.
This project is still made by a human trying his best.