# tesseract

![license](https://img.shields.io/badge/License-Apache_2.0-blue.svg)
![version](https://img.shields.io/badge/Version-0.0.0-darkred.svg)

An ECS-based Minecraft server toolkit written in Rust using Bevy.<br>
Build your own player experience!

## Overview

The main goal is to offer a basic infrastructure, and a toolkit for building Minecraft servers, with fully customizable
behavior. The every aspect of Minecraft is fully abstracted away. This way its also easy to write plugins, which can
affect the broader gameplay.

It's also notable that Tesseract uses a different terminology compared to most projects:

| Minecraft    | Tesseract             |                                                                |
|--------------|-----------------------|----------------------------------------------------------------|
| World        | Level Entity          | Every level is represented as an entity                        |
| Chunk        | Chunk Entity          | Every chunk is represented as an entity                        |
| Entity       | Actor Entity          | Actors are entities like everything else, but not the only one |
| Block        | (Entity-driven) Block | Every block variant can be represented as an entity            |
| Block Entity | Entity-driven Block   | Every block entity is represented as an entity                 |

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
