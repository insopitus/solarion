# Asset Credits

## Character model

`models/heroes/xbot_animated.glb`

- Mesh: **X Bot** from the **Sword and Shield Pack** by
  [Quaternius](https://quaternius.com) (CC0 1.0).
- Animations: the same pack's idle, walk, run, attack, slash, casting, death
  and impact clips (Mixamo rig).
- Local source: `downloaded-assets/models/Sword and Shield Pack/`

### Conversion pipeline

Bevy cannot import FBX, and Blender 5.2 rejects `X Bot.fbx` because it is the
old FBX 6.1 format. The model was therefore converted with
[FBX2glTF](https://github.com/facebookincubator/FBX2glTF) v0.9.7:

1. Convert `X Bot.fbx` to `conv.glb` (mesh + skin, no animation).
2. Convert each animation FBX (FBX 7.7) to its own glb.
3. Copy the animation channels into the mesh glb, remapping target nodes by
   bone name (`mixamorig:*`).
4. Zero the horizontal components of the `mixamorig:Hips` translation track so
   the mesh animates in place; the game moves the logical hero entity instead.
5. Rename the clips in this order: `idle`, `walk`, `run`, `attack`, `slash`,
   `casting`, `death`, `impact`. `src/combat/animation.rs` loads labels
   0, 2, 3 and 5.

## Floor texture

`textures/floor/ruby-trim-castle-tile-001.png`

- Source: [Free Game Assets](https://www.freegameassets.com/) (CC0)
- Local source: `downloaded-assets/textures/ruby-trim-castle-tile-001.png`
- Applied to the arena floor with repeat sampling (about 10 world units per
  texture repeat).

## Skill icons

See `icons/skills/README.md` for the Free Game Assets (CC0) spell icons and the
source filenames used for each skill.

## Other downloaded models

`downloaded-assets/models/Universal Base Characters[Standard]/` (Quaternius,
CC0) was used earlier as a static placeholder and is not referenced by the
game any more. Its glTF files can be reused for party variety later.
