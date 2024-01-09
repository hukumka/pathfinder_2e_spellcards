# Pathfinder 2e printable spell cards

## State:

Currently only very few spell have been added.

+ Cantrips: 5
+ Level 1: 9
+ Level 2-10: 0
+ Focus spells: 0

Goals: add all spells from remaster player core, but completness is currently low priority.
Though feel free to request extra spells in issues.

## Usage:

Write list of spells to render in following format:
```
cantrips/divine_lance.html
cantrips/needle_darts.html
level1/heal.html
level1/heal.html
```

Run command generating spellcards:

```
spellcard_generator < spells.txt > static/index.html
```

Open `static/index.html` in browser and select `Menu -> Print`.

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
