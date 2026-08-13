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


## Pass three: the outside of the house

- **`FLY_OUTSIDE=<deg>` exists because nothing else could see this.** Every
  viewpoint so far was inside the building or straight above it, and between
  them they could show every room and still never answer what the place looks
  like. The first exterior capture was a flat white lid: the house had no roof
  at all, only per-room ceiling slabs.
- **A gable roof, in boxes.** Two turned slabs per roof, fascia and soffit at
  each eave, and the triangular gable ends built as courses of siding cut off
  under the slope. The main house runs its ridge east-west; the garage's turns
  ninety degrees so its gable faces the drive, which is what this plan gets
  built with. Two faults found by capture: the slope rotations were sign-flipped
  so both planes tilted the wrong way, and cutting each gable course at its
  *top* edge left a triangular gap per course — eighteen shark's teeth along the
  rake. Cutting at the bottom edge buries the overshoot in the slab above.
- **A shelf unit was standing outside the house.** `shelves` swapped its own
  axes on top of the mapping its `dim` closure already did, so `along_x: false`
  still ran the boards along x: the garage's shelving came out three metres wide
  and stood a metre through the east wall. No interior capture could show this —
  from inside, furniture that has escaped just looks small.
- **A law for it.** Nothing that is not roof or ground may cross the outer face
  of the walls. Tested by shoving the shelf back out: 92 faults. The first
  version only checked solids whose *centre* was still indoors, which excused
  exactly the pieces that got furthest out — the same shape of hole as the
  traversal law's.
- **The plan view had been framing the lawn**, which runs forty metres past the
  house on every side, so the whole building sat in a thumbnail in the middle of
  a field. It frames on the building now.


## Pass four: doors, light, and the bathrooms

- **A front door.** The front elevation had a black hole in it. Stile-and-rail
  construction — two stiles, three rails, two recessed panels — built in the
  door's own frame and then swung, plus casing both faces, a threshold, a knob,
  a stoop and a step. Hung ajar: it is the only opening in the house that is not
  glazed shut, so closing it walls the fly in. The envelope law caught the first
  version swinging out into the garden.
- **Fixtures and bounce.** Every ceiling was a flat grey field, because the room
  lamps are downlights by design and nothing bounces, so the plaster was lit by
  ambient alone. There is a rose and a glowing diffuser in every room now —
  which is what `Solid` needed a `glow` term for — and a wide, dim spot from
  table height aimed straight up as a stand-in for the one bounce that matters.
  The fixture was two squares at forty-five degrees, which unions into an
  eight-pointed *star*; four crossed bars gives the octagon.
- **The bathrooms were the weakest rooms in the house.** The main bath is
  sixteen by twenty feet at this plan's scale, and it held a plain box for a
  basin, two boxes for a lavatory, and one small tub adrift in the middle of a
  tiled hall. It now has: a proper lavatory (foot, waisted pedestal, bowl, seat,
  cistern, raised lid), basins sunk into the vanity with mixer taps, a bath with
  a riser, head and glass screen, a linen press, a corner shower, towels on a
  rail, a mat, a basket, and tiling where a bathroom is actually tiled.
- **Corner viewpoints kept lying to me about layout.** Three separate rooms
  looked under-furnished from one corner and turned out to be fine — the laundry
  has had its washer and dryer all along. The whole-house plan is the honest
  view for judging arrangement; corners are for judging how a room reads.


## Pass five: the bedrooms

- Bare mattresses with two pillows on them, plain-box wardrobes, plain-box
  chests, and two entirely blank walls per room. Now: a duvet stopping short of
  the pillows with the sheet turned back over its edge and a throw across the
  foot; a wardrobe with a plinth, doors, a shadow gap and handles; a chest with
  real drawer fronts and pulls; a bedside lamp with an eight-sided shade; a desk
  and chair under the far wall in the children's rooms; pictures on the wall the
  bed does not use.
- **`chair` is a constructor now.** The kitchen had four written out inline,
  which is exactly the repeated recognisable form that should not be inline.
- **The window law fired again, on new work.** The wardrobe and both new
  pictures went straight over west-wall glass in two bedrooms out of three,
  because `clear_of_windows` only ever looked at the *north* wall. It takes a
  `Wall` now and every placement that needs a blank stretch asks for one.
  This is the third separate time this rule has caught something; it is by far
  the highest-value law in the file.


## Pass six: the doorways

- Every interior opening was a bare hole with square plaster edges. They all
  have a lining and an architrave both faces now, which is most of what makes a
  doorway read as part of a built house, and the two wide cased openings between
  the hall and the middle of the house get the same treatment without a leaf.
- **Seven interior doors, hung.** None of them shut: the traversal law fails the
  moment they do — a fly cannot work a handle — and a house with every internal
  door closed is a house nobody lives in. Each stands somewhere between wide
  open and just ajar, picked off its own position so a door is at the same angle
  every run, then **walked back until the leaf is not standing in a bed or a
  wardrobe**. The door gives way, because it is opening into a room that was
  furnished before it got there.
- Cost after this pass: 2245 draws, 15 meshes, 182 materials, 20 lights. Frame
  rate is still display-bound, capped or not.


## Pass seven: the grounds, and an exposure mistake

- **Drive, path, step and planting.** The house had been standing on an infinite
  lawn with no way up to either of its doors, which reads as a model of a house
  rather than a house. There is a bayed concrete drive out from under the garage
  door, a paved walk along the front, a spur to the step, and foundation
  planting between the windows. None of it is taller than a step, so the
  envelope law allows it outside; the shrubs get past by being `Stuff::Grass`,
  which is both what they are and the exemption they need.
- **The exterior had been rendering two and a half stops over.** The camera is
  set to `Exposure::INDOOR` (EV100 7) because a room lit by an 800-lumen bulb
  sits in the tens of lux and renders black at the daylight stop. Standing
  *outside* at that stop does the opposite: the lawn washed out to pale mint and
  so did everything planted in it. Three separate exterior passes — roof colour,
  wall colour, shrub colour — were judged on colours that were not the colours.
  `FLY_OUTSIDE` now meters for daylight. A camera meters for where it stands.


## Pass eight: cladding

- **Lap siding on every exterior wall**, emitted inside `wall_run` so it is cut
  around the openings the run already knows about. Which face is exterior is not
  a flag anyone has to remember to set: it is a probe thirty centimetres off
  each face asking `inside_envelope`. Boards are the cheapest thing that puts a
  scale on an elevation — they say how big the house is before you have looked
  at anything else in the frame.
- **Casing and sills round every window**, for the same reason: with the walls
  clad, a window became a rectangle punched in a run of boards, and siding is
  never left untrimmed.
- `TRIM_PROUD` now names how far anything nailed to a wall may stand out from
  it, and the envelope law allows exactly that. A sill is trim; a shelf is an
  escapee. The law caught the first sills at twenty-two centimetres.
- Cost: 2662 draws, 15 meshes, 230 materials, 20 lights.


## Pass nine: the ceiling

- Playing the game rather than posing a camera showed the obvious next gap: the
  fly starts *on the ceiling*, and the ceiling was a featureless plane taking
  half the frame. It matters more here than in most games — it is where the
  player begins and where a fly spends its time.
- **A ceiling fan in the great room**: downrod, canopy, motor, five pitched
  blades and a light kit, which is also where that room's lamp already was. It
  is what a ranch of this period has, and at fly scale it is five landing strips
  and a set of edges to walk round. `fixtures` skips the great room now, because
  the fan carries its own light.
- The eight-sided disc is a shared constructor now — the car's wheels, the
  ceiling roses, the lamp shade and the fan all use it.


## Pass ten: things on surfaces

- The House Quality Standard asks for restrained clutter, and every horizontal
  surface in the house was bare. The great room has cushions along the sofa back
  and a blanket over one arm, a stack of books, a mug and a remote on the coffee
  table, more books on the floor beside the sofa, and a pot plant in the corner.
  The kitchen has a fruit bowl, a chopping board, a kettle and two jars.
- **The first plant read as a hedge.** Six fat boxes at half the plant's height
  across is a shrub in a pot; nine thin ones on a visible stem is a plant. Same
  construction as the foundation planting outside, tuned the other way.
- New constructors: `books` (each volume a shade and a size off the one under
  it, top one out of square), `mug`, `pot_plant`.


## Pass eleven: a sweep, and what it found

- **Daylight through the shut garage door.** The sectional's panels stop three
  centimetres short of each joint and the reveal covering the joint was three
  tall, so every section had a pair of one-and-a-half-centimetre slots in it —
  a line of grass and sky right across a closed door. Eight-centimetre reveals.
- **The laundry's south wall was blank the full width of the room.** It has a
  hanging rail with clothes on it, an ironing board leaning where one always
  leans, a broom, a mop and a basket.
- **The washer and dryer read as one white cupboard.** Two pale carcasses side
  by side with nothing on their faces is a cupboard; a control fascia, a dial,
  a drum door with a glass port and a hinged door with a handle is a washer and
  a dryer. Same lesson as the toilet and the wardrobe: the carcass is never the
  part that identifies the thing.


## Pass twelve: photographs, switches, sockets

- **Photographs down the hall**, two or three to a stretch, stepped in height,
  placed in the gaps between the bedroom doors — worked out from where the
  doorways are rather than at hand-picked offsets. A frame over an architrave is
  the same mistake as one over a window.
- **Switch plates beside every internal doorway and sockets round the skirting.**
  The smallest thing that says a wall was built rather than extruded, and at fly
  scale they are landmarks: a switch is four body lengths across, standing a
  centimetre off an otherwise featureless plain.
- The clash test had to probe *off* the wall rather than at the plate. Testing
  at the plate rejects every plate in the house, because the wall is a solid too
  and the plate is screwed to it. First version placed six of sixty-eight.
- Cost: 2837 draws, 15 meshes, 278 materials, 20 lights, still display-bound.


## Pass thirteen: the entry

- The front door opens straight into the great room, so that room has to be the
  hall as well. A mat to wipe on, a rail with hooks, two coats on it and shoes
  kicked off underneath. The hooks go in the seventy-eight centimetres of wall
  between the door and the next window, which is the only place they fit — the
  window law would have said so otherwise, as it has three times now.

## Deferred

- Family simulation, needs, danger, death, objectives, progression, and HUD are
  outside the active house-and-lighting goal.
- Flight and camera redesign are deferred unless house validation reveals a
  specific regression that blocks traversal or inspection.
- Hinged door leaves. Openings are honest holes for now; a swinging door is its
  own piece of work and the greybox still has the only one.

## Next

- Then: fly-scale routes, undersides and landing surfaces; restrained clutter.
