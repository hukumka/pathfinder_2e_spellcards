This program uses trademarks and/or copyrights owned by Paizo Inc., used under Paizo's Fan Content Policy (paizo.com/licenses/fancontent). This program is not published, endorsed, or specifically approved by Paizo. For more information about Paizo Inc. and Paizo products, visit [paizo.com](https://paizo.com).

![IMG_20240509_181119](https://github.com/hukumka/pathfinder_2e_spellcards/assets/5196471/f7dc86fa-c163-4142-8933-b14588e2332c)

# Pathfinder 2e printable spell cards

Generate spell cards in fixed format to form spellcaster deck.

Currently supports two formats:
+ Normal: 6.3 cm x 8.8 cm
+ Double: 6.3 cm x 17.6 cm

Format is chosen automatically. Normal is default, but for spells that do not fit generator falls back to double format. 

![image](https://github.com/hukumka/pathfinder_2e_spellcards/assets/5196471/bea56a04-cf04-47f8-b3d1-44f80463f2aa)

Now with GUI for spell selection!

![image](https://github.com/hukumka/pathfinder_2e_spellcards/assets/5196471/485d0066-3a46-4af0-8331-dd48c761db4e)


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

Select spells you need by using search. Then export as pdf.

## Builing from source

1. [Install rust toolchain](https://rustup.rs/)

2. Install gtk packages

```
libgtk-4-dev
gtk4
glib2.0
pandgo1.0
freetype
```

3. Clone repository
```
git clone https://github.com/hukumka/pathfinder_2e_spellcards
cd pathfinder_2e_spellcards
```

4.Build:
```
cargo build --release
```

Binary will be in `target/release/` directory.

## Built from source (Nix)

Requires flake support enabled.

```
nix build github:hukumka/pathfinder_2e_spellcards
```

## Development enviroment (Nix)

This project provides ready development enviroment with `cargo rust and rust-analyzer` installed. To enter enviroment, in project directory run:

```
nix develop
```


