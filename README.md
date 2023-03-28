# tesseract

![license](https://img.shields.io/badge/License-Apache_2.0-blue.svg)
![version](https://img.shields.io/badge/Version-0.0.0-darkred.svg)

An ECS-based Minecraft server toolkit written in Rust using Bevy.

Build your own player experience!

## Current Status

- PersistencePlugin:
    - multiple levels
    - player loading (position, rotation)
    - chunk loading (block states, biomes)
- ReplicationPlugin:
    - encryption
    - compression
    - online mode
    - replicating chunks (delta, early, late)
    - replicating actors (delta, early, late, across chunks)
