# Autonomous Development Log

## Active Goal

Continuously build, furnish, light, inspect, and improve the entire house as a
procedurally generated Rust and Bevy environment. Create the architecture and all
props mathematically inside the game. Continue choosing the next meaningful
visual, structural, traversal, or performance improvement until Brett interrupts
the session. Do not treat one room or prop as completion.

## Baseline

- Flight feel and controls are approved and are not the present design target.
- The project has custom oriented-box collision and uses centimetres as units.
- Existing imported-house support, automatic room detection, and automatic room
  lights provide useful reference behavior, but Opificium is not part of this
  active goal.
- Existing capture switches can produce delayed screenshots and a plan view.

## Readings of the brief

- **Habitable means living space.** The fifteen-foot minimum applies to the great
  room, the kitchen and the three bedrooms. Bathrooms, laundry, closets, the hall
  and the garage are service space and are not held to it. Brett confirmed this
  reading. It is also the only reading under which the posted plan survives, its
  laundry being six foot eight deep.
- **The posted floor plan is the plan.** Brett asked that the house match the
  three-bedroom ranch he posted, within the goal's constraints. Its bedrooms are
  11'-6", so the whole drawing is scaled by exactly `15 / 11.5` rather than having
  individual rooms stretched. Every proportion survives; the tightest bedroom
  lands on the minimum precisely.

## Completed

- **`house.rs`: the plan, generated.** Authored in feet straight off the posted
  drawing so the two can be compared line by line, then scaled once. Three rooms
  down the left (bed 3, bath, bed 2), a hall the depth of the house, an open
  kitchen-and-great-room through the middle, laundry/main bath/main bedroom down
  the right, two-car garage on the end. 2703 x 1372 cm overall.
- **`wall_run`**, the only construction primitive: a straight run with holes in
  it. A doorway is a *gap described once* rather than four boxes to be re-derived
  whenever a room moves. Windows are the same call with a sill.
- **`audit`** measures the built result on every run rather than trusting the
  plan's numbers, and errors if a habitable room is under fifteen feet or if the
  first thing over a room's middle is not the ceiling. It caught its own scale
  factor: `11.5 * (15/11.5) * 30.48` is 457.19998 in `f32`, so the check needed a
  half-millimetre tolerance.
- **Authored lighting**, one fixture per room by use plus a low afternoon sun.
- **Glazing**: real panes in every window, landable and bumpable.

## Validation

- Plan capture (`FLY_PLAN=1`) confirms the built house matches the posted
  drawing's arrangement: left column, hall, open middle, right column, garage.
- `audit` passes: 10 rooms, 5 habitable, all at or over 457.2 cm, ceilings 274.32.
- Three consecutive captures now return identical mean luminance (91.8), where
  before they alternated between an image and solid black.

## Research / decisions

- **Bare point lights cannot be ceiling fixtures.** A point light 14 cm under the
  ceiling puts on the order of ten million lux on it — every ceiling rendered
  pure white. Winding it down would have unlit the floor, three metres further
  away, by the square of the distance. Room lights are now **spots aimed
  straight down**, which is what a recessed downlight physically is: it lights
  the floor and lower walls and leaves the plaster it sits in alone.
- **Ambient is standing in for bounce light** and has to be high for a "fill",
  because nothing in this renderer bounces. Household-accurate lumens give
  household-accurate dimness — a real 1500-lumen bulb at 2.6 m is about 18 lux —
  so the fixtures are deliberately far brighter than real bulbs.
- **Captures went through the window and had to stop.** Reading back a window's
  swapchain needs the compositor to have drawn it, and macOS does not draw an
  unfocused window: the result is a frame that is solid black *including its
  background*, with no error logged. It was mistaken for a lighting bug three
  times in one session. `capture.rs` now renders to an offscreen image, as Flat
  Earth Simulator does for the same reason. `FLY_CAPTURE_SIZE=WxH` sizes it.


## Pass two: the garage, and three faults it exposed

- **Cost, measured.** 1561 draws, 1559 meshes, 1549 materials — a mesh and a
  material per box, for a building whose whole vocabulary is one shape and about
  thirty colours. That defeats batching: the renderer groups draws by material.
  One shared unit cube with a scale on it, and a palette keyed on quantised
  colour, took it to 15 meshes and 111 materials with no visible change. Frame
  rate is display-bound at 60 with `FLY_UNCAPPED=1` as much as without, so the
  house is not the limiter and no further optimisation is justified yet.
- **The curtains were inside the walls.** Hung on the wall's centreline, which is
  where the *opening* is: an 8 cm panel inside a 20 cm wall. Every window had
  them and not one showed. They hang proud of the inside face now.
- **A real car.** The garage held a single slate box. It now has a car built the
  way the house is: sill, shoulder, decks, glasshouse, seats and a dash behind
  the glass, octagonal wheels made of four crossed boxes, lamps, bumpers, plate.
  Plus a sectional door with a glazed top panel, because a garage with a hole in
  the wall is a carport.
- **The house had a slot running round the garage.** The garage slab is a step
  down at -6 cm and every wall started at 0, so there was a six-centimetre gap
  with daylight and grass showing through it. `wall_run` now lays a footing to
  -30 under every run; above the garage floor it reads as the stem wall it is.
- **Curtains ask what is standing there.** `dress_the_windows` runs after the
  rooms are furnished and shortens a panel to sill length when a full drop would
  pass through a worktop. The kitchen was drawing curtains through its counters.
- **Corner viewpoints step clear.** Two garage captures came back as a close-up
  of the back of a shelf unit; `FLY_ROOM=<room>:<corner>` now walks the camera
  along its own sightline until it is 75 cm clear of everything solid.

## Deferred

- Family simulation, needs, danger, death, objectives, progression, and HUD are
  outside the active house-and-lighting goal.
- Flight and camera redesign are deferred unless house validation reveals a
  specific regression that blocks traversal or inspection.
- Hinged door leaves. Openings are honest holes for now; a swinging door is its
  own piece of work and the greybox still has the only one.

## Next

- **A roof.** The house has never been looked at from outside, and there is
  nothing on top of it. This is the largest remaining structural gap.
- Ceilings read dead flat: nothing bounces, and the room spots point straight
  down at the floor by design. Wants a visible fixture at least.
- Service rooms are furnished but thin — the laundry has a blank long wall.
- Then: fly-scale routes, undersides and landing surfaces; restrained clutter.
