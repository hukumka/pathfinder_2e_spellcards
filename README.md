# Pathfinder 2e printable spell cards

Generate spell cards in fixed format to form spellcaster deck.

Currently supports two formats:
+ Normal: 6.3 cm x 8.8 cm
+ Double: 6.3 cm x 17.6 cm

Format is chosen automatically. Normal is default, but for spells that do not fit generator falls back to double format. 

![image](https://github.com/hukumka/pathfinder_2e_spellcards/assets/5196471/bea56a04-cf04-47f8-b3d1-44f80463f2aa)

## State:

Capable of rendering most spells.

Spells that currently cannot be rendered due to being very long, and not fitting on double card format:

+ Umbral Extraction
+ Chromatic Wall
+ Mad Monkeys
+ Summon Elemental Herald
+ Element Embodied
+ Ooze Form
+ Summon Ancient Fleshforged
+ Illusory Creature
+ Summon Kaiju
+ Avatar

## Usage:

Write list of spells to render in following format, and save in `spells.txt`:
```
divine lance
needle darts
heal
heal
```

Run command generating spellcards:

```
spellcard_generator
```

Output is stored in `output.pdf`


## Builing from source

1. [Install rust toolchain](https://rustup.rs/)

2. Clone repository
```
git clone https://github.com/hukumka/pathfinder_2e_spellcards
cd pathfinder_2e_spellcards
```

3.Build:
```
cargo build --release
```

Binary will be in `target/release/` directory.
