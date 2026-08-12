# Fly on the Wall — flight test

This is not the game. It is the first spike, and it exists to answer one question
before anything else gets built:

> **Is being a fly worth doing?**

Two rooms, a door, and a fly. No family, no hunger, no danger, no night, no
scent, no death. All of those are downstream of movement feeling right, and none
of them can rescue it if it doesn't.

Double-click **`Play.command`**, or:

```bash
cargo run
```

Debug profile on purpose — dependencies already build at `opt-level = 3`, so it
is fully playable and starts immediately. `--release` would trigger a long
rebuild to speed up thirty boxes and one insect.

## The pass test

Pick one and hold the build to it, or this runs forever:

> From a rest on the ceiling of the living room, take off, thread the **ajar**
> door, and land upside down under the kitchen cabinets — in one continuous
> motion, without thinking about the controls.

Two more, worth as much as the first:

- Hand it to someone cold. Can they do it inside a minute?
- Ask them which room they're in without letting them look at the readout.

## Controls

| | |
|---|---|
| Mouse | aim |
| `W` `A` `S` `D` | `W` thrust, `S` **brake** (a fly cannot fly backwards), `A` `D` sideslip. All four crawl when perched — including backwards, which flies really do |
| `Space` | climb, or take off from a surface |
| `Left Ctrl` | descend |
| right mouse / `F` | **hold to land.** Contact only sticks while it is held; let go and the fly grazes off what it clips |
| `E` | cycle the door: closed → ajar → open |
| `[` `]` | narrow or widen the ajar gap, a millimetre a press |
| `Q` | chase camera ↔ first person |
| `R` | keep the room upright ↔ keep the fly upright |
| `F3` | the readout |
| `F12` | save a screenshot |
| `Esc` | release the mouse |

## Switches

| | |
|---|---|
| `FLY_HOUSE=<name>` | pick a building drawn in Opificium. A bare name is looked up in `assets/buildings/`, which is where the bench's `install` puts them. **A drawn house is the default** when one is installed; `FLY_HOUSE=greybox` asks for the two-room movement test back |
| `FLY_MODEL=glb` | use `assets/fly.glb` instead of the procedural fly |
| `FLY_INSPECT=<deg>` | park the camera close to the fly at that azimuth — 0 behind, 90 side-on, 180 head-on — and stand it on the living room floor. For looking at the model, not playing. |
| `FLY_PLAN=1` | look straight down at the whole house with the roof dropped — the only sane way to check a drawn floor plan |
| `FLY_CAPTURE=<path>` | render for a moment, save a frame there, exit |
| `FLY_CAPTURE_DELAY=<s>` | move the shutter (default 4) |

```bash
FLY_INSPECT=90 FLY_CAPTURE=side.png cargo run
FLY_HOUSE=ranch cargo run
```

## Drawing a house

The floor plan is drawn in [Opificium](https://github.com/Baz-Studios-LLC/Opificium),
the studio's bench, not typed into `world.rs`. The project lives at
`opificium/` in this repo:

```bash
OPIFICIUM_PROJECT="$PWD/opificium" opificium
```

Draw, bake, and the manifest's `install` carries the finished building to
`assets/buildings/`. Then `FLY_HOUSE=<name> cargo run`.

**The project sits beside `assets/`, not inside it**, and that is deliberate.
The asset folder is copied verbatim into the shipped bundle, and `file_watcher`
has the running game watching it for changes — a bench autosaving in there would
both post the workshop to every player and reload the game mid-write. `install`
being an explicit step is the point: two programs writing and watching one tree
is how you get a bug that turns up once a month.

Blueprints under `opificium/out/buildings/` are tracked, because they are the
floor plan's source and very diffable. The bake's intermediate output is not.

You start hanging upside down under the living room ceiling. Falling off it is
the intended first thirty seconds.

## The three things this build is actually asking

**1. What should happen to the horizon when you land on a ceiling?** — `R`

`WorldUp` keeps the room upright and draws the fly inverted. `BodyUp` rolls with
the fly, so the room turns over. Both are defensible, they feel completely
different, and there was never any way to settle it from a desk. Land on a few
ceilings in each.

**2. How close is too close?** — `Q`

First person is the honest answer to what a fly sees and a strong candidate for
making people ill. The chase cam is the default because it's the safe one, not
because it's the right one.

**3. How wide is "ajar"?** — `E`, then `[` and `]`

Ships at 10°, which is a 1.2 cm slot — two fly-lengths. `[` and `]` move it half
a degree a press and the readout shows both the angle and the resulting slot.

Worth knowing why it isn't smaller: a hinged door covers its whole opening at
*every* angle, so what you squeeze through is the slot between the door's free
edge and the far jamb — `width × (1 − cos θ)`, which is quadratic and stays
useless for a surprisingly long time. A door standing 2° open has a
four-tenths-of-a-millimetre gap. Ten degrees is roughly where the free edge also
swings clear of the wall's thickness, so both constrictions open together.

## What to watch for while playing

- **Walking is a real verb, not an idle pose.** Close the door and get into the
  kitchen anyway. There's a 1.2 cm gap under it and that's the only way through.
  Same for the 15 cm behind the fridge — unreachable in flight, trivial on foot.
- **Crawl from the floor up a wall and onto the ceiling** without taking off.
  Both edge cases are implemented (concave, and convex around a lip); if either
  drops you, that's a bug and not a design decision.
- **Stand on the door and press `E`.** The perch is stored in the door's frame,
  so the fly rides it.
- **Listen to the wingbeat rather than reading the speed.** It's the only gauge
  in the game and it's meant to be enough. Landing cuts it to silence, which is
  the loudest feedback in the build.
- **Watch the legs.** They tuck in flight, splay when perched, and reach the
  moment you hold to land — as does a nose-up flare of the whole body. Those are
  the entire no-HUD thesis in one animation, and they're honest: they show
  because you asked, not because the code guessed. If nobody notices, they need
  to be bigger.
- **Fly past something at speed without holding the button.** You should graze
  off it and keep going. Then do it again holding right mouse, and you should
  end up stuck to it. That difference is the whole control scheme.
- **Sweep the mouse slowly while holding `W`.** The fly should *not* follow in a
  curve. It holds its course, then snaps across in about fifty milliseconds, then
  holds again — a real fly's path is a polyline, not an arc, and this is the one
  thing here that no other game does. `SACCADE_ARC` in `fly.rs` is the dial;
  set it to 0 for the ordinary smooth curve and feel the difference.

## What this build cannot tell you

A movement model with nothing at stake can feel excellent and still be wrong.
Pressure changes how a control scheme reads, and there is no pressure here.
*Flight is solved* stays provisional until something in the house is trying to
kill you.

## Notes on the build

- **One unit is one centimetre.** The fly is just under a unit; a countertop is
  90; a room is 500. Rooms are authored in metres via `world::m()`. This is the
  decision the rest of the project inherits — metres would put the fly's
  collision radius at 0.003 and every engine default is tuned against that.
- **No physics engine.** The world is two dozen boxes and the only queries needed
  are a slab raycast and a closest-point clamp. Both are in `world.rs` and are
  shorter than the integration would have been.
- **No Ordo yet.** Its job in this game is the layer *outside* play — the death
  card, the field notes, the obituary — and none of that exists. The readout on
  `F3` is instrumentation that gets deleted, and taking a UI kit for it would put
  the dependency in the tree ahead of the thing it's for.
- **Three lighting traps, all structural**, recorded because each one looks like
  a tuning problem and none of them is. A sealed room cannot be lit from
  outside — a directional light over a closed box contributes *nothing* indoors,
  so every surface drew pure ambient with no shading at all. That is what the
  window is for. Bevy's lighting is physical and assumes one unit is one metre,
  so at a centimetre to the unit a ceiling bulb reads as 250 *metres* away and
  inverse-square annihilates it; lumens need multiplying by 10,000. And the
  camera's default exposure is `BLENDER`, EV100 9.7, calibrated for daylight — a
  room lit in the tens of lux renders black against it. `Exposure::INDOOR` was
  worth more than any amount of fiddling with the lights.
- **`jpeg` is not a default Bevy feature.** The glTF's textures are JPEG, so
  without it the whole model fails to load with `invalid image mime type` and the
  fly is simply absent.

| | |
|---|---|
| `world.rs` | boxes, geometry queries, the two rooms, the hinged door |
| `fly.rs` | flight, landing, surface walking — the file the spike is for |
| `body.rs` | the blocky fly, wing blur, leg poses |
| `camera.rs` | chase and first person, and the two roll conventions |
| `wingbeat.rs` | synthesised buzz, pitch driven by effort |
| `debug.rs` | the readout, on `F3`, which does not survive the spike |
