# tesseract

![license](https://img.shields.io/badge/License-Apache_2.0-blue.svg)
![version](https://img.shields.io/badge/Version-0.0.0-darkred.svg)

An ECS-based Minecraft server written in Rust using Bevy.

The core concept is that everything is an entity, this means even the level itself is an
entity, and each individual chunk, blocks, items, etc. and can be interacted with in an ECS-based manner.

This all is realizable thanks to hierarchies, each level has chunks as children,
each chunk has "actors" (Minecraft's Entities) as children, and instancing+referencing (not every stone for example
has to be its own entity, only if the behavior is different "special stone")

All this enables great flexibility, with superior performance.

## Current Status

- persistence
  - one level
  - synchronous chunk loading (block states, biomes)
- replication
  - encryption
  - compression
  - online mode
  - replicating chunks (early, late)
  - replicating actors (early, late, across chunks)
