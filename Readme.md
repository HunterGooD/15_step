# 15 step 
This repo for game in 15 steps. 15 beginner steps in gamedev </br>
The steps do not claim to be correct when developing games. They exist for enthusiasts who just wanted to create their own game. </br>

Project structure borrowed from the [bevy_template](https://github.com/NiklasEi/bevy_game_template) repository.

## 1 Decide to write a game
The idea of a word game. The idea of a game in a couple of words.
Plans will be minimal and will continue to add new features to test, the first game should not be difficult to develop. </br>

- Game 

The game will present a sequence of levels that are procedurally generated, in every 3 rooms the player will encounter a boss. The effects of some items will also have some kind of generation system.
At the locations, artifacts will come across that will strengthen the hero. Some items will allow you to change class while maintaining the basic characteristics of the initial class

- Player

The player will be presented with a choice of 3 classes

- Wizard 
- Warrior
- Archer

The player can carry a certain number of items with him. Or an infinite number in fun mode.

- Enemy 

There will be several types of enemies. They will have their own zest, indicating the strength or ability of a given enemy. Bosses can look like upgraded versions of regular enemies.

- Artifacts

Items will play an important role in the gameplay, as this is almost the only thing that can make the player stronger. There will also be 2 active items that you will need to pick up wisely.

## 2 Choose language and platform

The programming language Rust was chosen as the development language. Among the libraries for games, one of the most popular libraries with a good community was chosen - Bevy. For the physics of the game, the bevy_rapier2d library was chosen. For user input, the leafwing-input-manager library was chosen as the most convenient way to accept user input. </br> 
In the future, the list of libraries can be expanded

## 3 Opportunity check

This step is practical. It needs to define user input and output, while doing everything without configuration files. Make debug information for further debugging. Implement character movement, add a couple of platforms.

## 4 Map

You need to select map tiles, collisions, backgrounds and other decorations, you can also interact with some elements of the map such as doors and so on. At this stage, map generation is not needed yet, everything can be hardcoded.

## 5 Saving/Loading - SKIPPED

(First: Note that you can initially do away with save and load entirely - a feature not present in many early implementations - and generate your dungeons instead of hard-coding them, which is arguably a more roguelike approach!)
I skipped this step due to the complexity of implementing collider serialization from the bevy_rapier2d library. There is a method how it could be implemented, but this is too bad code.

## 6 Enemy

Implement monsters, add simple AI. For example, chase the player if he is in sight or walk back and forth while waiting.

## 7 Interaction

At this stage, you need to add the characteristics for your creatures. I will add a characteristic as needed, and not because it “looks cool”. </br>
Also at this stage it is worth improving the "AI" of creatures, noticing each other - push, attack, etc. </br>
Implement and test the combat system - no hardware yet, just hardcode the values. </br>
