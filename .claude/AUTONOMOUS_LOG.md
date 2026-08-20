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


## Pass fourteen: the kitchen's faces

- The fridge, the cooker and the wall cabinets were all carcasses with nothing
  on their fronts — the same fault as the toilet, the wardrobe, the chest and
  the washer, and by now clearly the single most common one in this file. A
  carcass says how big a thing is; the face says what it is.
- Fridge over freezer with a shadow gap and a handle down the same side of each.
  A hob with four rings, a control fascia with four knobs, an oven door with a
  glass window and a bar handle. Doors and handles on the wall units.


## Pass fifteen: gutters

- A gutter hung on every fascia and five downpipes with brackets and shoes. It
  is the last thing an eave was missing, and at fly scale a gutter is a
  forty-metre trough with a lip on it, out of the weather and out of sight —
  which is exactly the kind of route the quality standard means by "sheltered
  spaces".


## Pass sixteen: the bookshelf

- Every shelf held five evenly spaced blocks alternating between two colours,
  which is a comb, not a bookshelf. What makes a shelf read is uneven widths, a
  couple of leaners, a stack lying flat, and a gap where somebody took something
  out. The row now walks along the board placing whichever of those the position
  calls for, so it is different on every shelf and the same on every run.


## Pass seventeen: the workbench wall

- A pegboard over the garage bench with hammers, a saw, spanners and a level
  hung crooked on it, and paint tins underneath. The wall above a workbench is
  the one wall in a house that is never blank — and it was the last blank one.


## Pass eighteen: texture

- Brett asked for refinement rather than more objects, and the loudest thing
  about how this house looked was that **every surface in it was a flat
  colour**. There is a sixty-four-pixel procedural texture per material now:
  tooth on plaster, grain running the length of a floorboard, a weave with
  slubs in it on fabric, aggregate in stone, a brushed direction on metal.
- They *modulate* the palette rather than replacing it — kept close to white on
  purpose — so every colour decision made in the previous seventeen passes
  still holds.
- The tiling rate has to be part of the material key. One material serves every
  box that looks alike, and without it a floorboard and a table leg share a
  material and the leg gets a floorboard's worth of grain on it. Materials
  359 -> 616 as a result, which is the price of the thing working.


## Pass nineteen: outside the window

- Brett asked for a street, other houses, and yards. Every room in this house
  had looked out onto an unbroken green plane to the horizon, which is the one
  thing that gave away that the building was a model of a house rather than a
  house in a place.
- Two-lane road with a dashed centre line, kerbs and pavements both sides;
  five neighbouring houses built with the *same* roof constructor so the ridges
  and eaves match and the street reads as one street; drives and paths to each;
  street trees; a mailbox at the kerb; and out the back a fence, a shed, a
  patio and a washing line.
- **The fly does not go outside.** Brett's call, and it settles two things: the
  outside only needs a silhouette and a roof line, because it is only ever seen
  through glass; and the front door and the raised kitchen sash both had to be
  shut. That is the better game — the street is right there and can never be
  reached, which is exactly what a window means to a fly.
- `Solid.outdoors` replaces two inferences that had stopped being safe: planting
  was recognised by its material and paving by being no taller than a step.
  Neither survives a house across the road.
- Two things the first capture killed: trees built as a stack of wide flat boxes
  read as a pile of crates painted green — eighteen small lumps on a rough
  sphere read as a canopy. And the ground's grain was tiling sixty times over a
  hundred and eighty metres, which put visible blocks the size of a car on the
  lawn.


## Pass twenty: the siding, twice wrong

The north elevation came back stippled all over, and it took two wrong
diagnoses to get to what it was.

1. *Shadow acne* — it is not. A north wall with the sun in the south-west has
   `N·L < 0` and never samples the shadow map at all.
2. *Exposed board tops* — closer. Each course had a three-centimetre gap above
   it, and the top face of a board points up, at the sun. A hundred lit slivers
   three centimetres wide, seen at a grazing angle, alias into exactly what
   shadow acne looks like. Closing the gap by lapping the boards fixed that
   and immediately introduced the third thing:
3. *Z-fighting*. Overlapping boards at the same depth put two front faces on
   one plane, which stipples in three-centimetre bands at every course line.

The answer was to stop building lap siding flat. A lapped board's bottom edge
stands proud of its top — that is what throws the shadow line that makes siding
read as siding. Tilting each board fifteen hundredths of a radian covers the
tops *and* puts no two faces on a plane, and the sunlit elevation gained real
relief into the bargain.


## Pass twenty-one: taking the corners off

- Everything soft in the house was a hard-edged box, and a cushion with eight
  sharp corners is the one shape upholstery never has. `soft` builds a box as
  three crossed slabs, each full length on one axis and inset on the other two:
  the corners go for the price of two extra boxes.
- Applied to the sofa's seat and back, the bed's pillows, duvet and throw, the
  towels, and the car's sill, shoulder and roofline — a car has no square corner
  anywhere on it and the roofline is the edge the eye checks first.
- The sofa's arms are **rolled**: four crossed bars round a cylinder. A square
  arm is the giveaway that a sofa was made of boxes.
- The blanket over the arm read as a plank until it got the part that hangs
  down the outside. That is what makes a blanket a blanket.


## Pass twenty-two: mouldings, and a texture bug they exposed

- Skirting and cornice were single square sections — a stripe of paint where
  the wall meets the floor and another where it meets the ceiling. Both are
  stepped now: a board with a bead set back from its face, and a cornice in two
  planes. The step is what catches a line of light all the way round a room,
  and it is in every room in the house for two extra boxes a run.
- Doubling the trim made a texture bug obvious. The tiling rate was keyed off a
  solid's **largest** extent, so a skirting board four metres long and nine
  centimetres tall repeated its grain eight times across those nine
  centimetres: every moulding in the house looked like corduroy. Keyed off the
  **median** extent instead — the dimension that actually governs how a face
  reads — it is clean, and materials fell from 745 to 470 as a side effect.
- Wood grain amplitude halved. Most of the wood in this house is painted trim,
  and painted joinery does not have a floorboard's grain on it.


## Pass twenty-three: curtains in folds

- One slab of fabric per curtain, and a pair at every window in the house. Five
  narrow strips with every other one pushed forward is a gather: the front
  faces catch the light and the ones behind fall into shadow, which is all a
  fold is from across a room.
- The first version also tapered the outer strips, which gave the curtain a
  staircase down its edge — worse than the plank it replaced. Every strip runs
  the full drop; the depth alternation does the work on its own.
- The window law caught the tilted siding on the way past: a lapped board is
  taller than its pitch, so centring it on the course line hung its bottom edge
  three centimetres low, and above a window that is three centimetres of
  cladding inside the opening. Boards sit *on* their line now.


## Pass twenty-four: paint

- Every wall in the house was the same grey, which is the strongest single
  reason the rooms all read alike however differently they were furnished. A
  family does not paint a house one colour: the living rooms go warm, the wet
  rooms go cool, and the children get to choose.
- Done as a skin half a centimetre proud of the plaster, coloured by the room
  it faces. Which room that is comes from the same trick the cladding uses to
  find the outdoors — a probe thirty centimetres off the face, asking `room_at`.
  A wall between two rooms gets painted twice, once on each side, which is what
  happens in a house.


## Pass twenty-five: the two biggest surfaces

- **Ceilings painted white.** Not decoration: the ceiling is the surface the
  bounce fill is aimed at, so it is the one place in a room where a lighter
  colour buys light everywhere else.
- **Floorboards to mid oak** from something close to walnut. A dark floor under
  a dark rug made the bottom half of every interior one brown mass, and the
  floor is the largest surface in any room — it sets the key for everything
  standing on it.


## Pass twenty-six: clutter, properly

Brett: *"This reads like an empty hotel room. Houses have clutter."* He is right —
one picture centred on each wall and nothing on any surface is a show home.

- **A gallery wall**, five frames at four sizes not quite aligned, over the
  media unit — which is the wall the room is looked at across. The first
  version put it on the wall *behind* the camera, which is a gallery nobody
  sees, and painted the frame and the image the same darkness, which reads as
  five brown blocks. Frame, pale mount, image: it is the mount that says
  "picture".
- A standard lamp in the corner, a basket of magazines by the sofa, a soundbar
  and a photograph and a plant on the media unit, a dish and a candle on the
  coffee table, and a couple of things on the floor nobody has put away.


## Pass twenty-seven: clutter everywhere, and a picture bug it exposed

- Books, a clock and a plant on the bedroom nightstands; toy blocks across a
  child's floor; clothes over the chair in the main bedroom; a knife block, a
  tea towel and a notice board in the kitchen; a roll, a bin and a toothbrush
  mug in the bathrooms; a bowl for keys and an umbrella in the hall.
- **Every framed thing in the house was rendering as a dark block**, and the
  reason was a lie in a comment. `picture` claimed to work out which way was
  into the room and then always used +z, so anything hung on a south or east
  wall had its mount and image buried in the plaster. It faces the middle of
  the room it is in now — probing a fixed distance does not work, because a
  partition is twelve centimetres thick and a picture hangs eight off the face,
  so a long probe finds the room next door and a short one is inside this room
  on both sides.
- The north windows were filled sill-to-head with fence. It is nine and a half
  metres out now, with two houses backing onto the garden behind it — their
  roofs over the fence are most of what a north-facing window in a street like
  this ever shows.


## Pass twenty-eight: a stripe on the wall

Faint vertical banding across a patch of one wall. Three wrong answers before
the right one, each ruled out by a test rather than a guess:

1. *The paint skin z-fighting with the wall.* Plausible — it sat half a
   centimetre proud. Splitting each wall into two painted halves removes the
   coplanar pair entirely, and the banding was unchanged. (Kept anyway: it is a
   better construction and the same box count.)
2. *Shadow acne from the downlight*, which grazes every wall in its room and
   would explain a patch bounded by the light's cone. Raised the spot's depth
   and normal bias for centimetre scale. Unchanged. (Kept: the defaults are
   tuned for metres.)
3. Turning `base_color_texture` off for one run: **gone**. It was the texture.

Vertical stripes mean a texture that varies along one axis only, and the wood
grain did — `hash2(0, y)`. Which way that stripe lands depends on how the face
happens to be oriented, and Bevy's cuboid UVs put v across the horizontal on
some faces. Wood grain is mostly isotropic now, which it can afford to be:
almost all the wood in this house is painted trim.


## Pass twenty-nine: glass, and pictures off the wall

- **Reflective glass.** Brett asked for it and he was right: a pane with nothing
  to bounce is a tinted hole. Reflectance up to 1.0, and a thirty-pixel-a-face
  procedural sky cubemap on the camera so there is something to reflect from any
  angle rather than one sun glint from one angle.
  - Intensity had to come down from 260 to 85. At 260 the sky's blue washed
    every warm wall in the house cold. And the cubemap's *down* face was grass
    green, which turned every ceiling olive — a ceiling faces down, so that is
    the face it samples. Neutral floor-grey instead.
- **Pictures were hanging off the wall.** Brett found it flying. `Room` bounds
  are the plan's centrelines, not the finished faces, so a caller writing
  `r.min.x + 7` lands one centimetre off the plaster on a twelve-thick partition
  and three centimetres *inside* a twenty-thick exterior wall. A centimetre is
  nothing to a person and four body lengths to a fly.
  - Two failed fixes first. Snapping to the room bound put every picture in the
    middle of the wall. Then the frame was still being drawn *before* the snap,
    so it floated while its mount and image moved flush behind it — dark blocks
    again.
  - What works: ignore the caller's offset and the wall's thickness both, and
    hang on the nearest large room-facing surface — *bounded to a nudge*.
    Unbounded, two pictures snapped to surfaces metres away and reappeared in a
    kitchen window, which the window law caught immediately.
  - Mount and image insets are proportional now. A fixed twenty-centimetre
    border is a mount on a big frame and a letterbox slot on a small one.


## Pass thirty: a layout review, room by room

Judged from the plan view, which is the only honest way to look at arrangement.

- **The great room was the bad one.** Sofa against one wall, television against
  the other: nineteen feet apart, with a void the size of a bedroom between
  them. The seating group is floated now — sofa, rug, coffee table and lamp
  gathered into a zone about ten feet across — and the floor left over becomes
  the route between the hall's two openings, which it needed anyway.
- The lamp and basket had to move again once the sofa did: they ended up
  standing in that route. A standard lamp planted in a walkway is a thing
  people walk into.
- **The western third then read as bare circulation with a full bookshelf
  standing in it and nothing to sit on.** It has a reading chair, a side table
  and a lamp now. A bookshelf nobody can read at is storage.
- **The two children's bedrooms were identical** — bed, wardrobe, chest and
  desk in the same places, which is one room built twice. Bedroom two is
  mirrored: wardrobe on the east wall, chest on the west, desk at the other end
  of its clear run. `clear_of_windows_on` grew an `East` case for it.
- The plan view was also framing the *neighbours'* roofs, because `gable_roof`
  marks its work as roof and nothing else, and the plan frames on everything
  that is not outdoors.


## Pass thirty-one: what Brett found by flying (v0.2.1)

Three faults, all found by playing and none by any capture I had taken.

- **A gallery hanging in mid-air over the dining table.** It was hung on the
  great room's "north wall", and there is no wall there — the kitchen and the
  great room are one open volume. The snap found nothing behind the frames and
  left them where they were asked for.
- **Pictures inside doorways.** `clear_of_windows_on` was named for the only
  thing it knew about, and that was the bug: a gallery spread across a door
  opening and a notice board hung inside a cased one, because neither is a
  window. It asks `house::all_openings` now — windows, interior doors, cased
  openings, the front door and the vehicle door.
- **A wardrobe standing inside a bed.** The bed takes the widest clear run on
  the north wall and the wardrobe takes the widest on the side wall, and on
  adjacent walls those two runs meet in the same corner. The bed is built
  first, so the wardrobe gives way: it tries the middle of its run, then each
  end of the room, and takes the first that is clear.
- And a photograph floating eighteen centimetres above the media unit, because
  it was going through `picture`, which hangs things on walls. It stands on its
  own foot now.

**The law that made all of this findable:** `picture` refuses to build when
there is no wall behind it, and says which one and where. Four came out on the
first run — the exact four Brett was looking at.


## Pass thirty-two: a law that nothing floats

Brett found three faults by flying that no capture of mine had shown, and the
right response was not to fix three things — it was to make the class of fault
impossible to ship unseen.

**Every solid in the house must touch another one.** That is the whole rule.
Bucketed on a metre grid, because otherwise it is twenty-two million pairs.
Rotated solids are skipped: an axis-aligned half is a lie for a tilted siding
board or a car wheel.

It found **twenty-five floating objects on the first run**, in four classes I
had no idea about:

- Every **chest-of-drawers handle** in the house, a carcass-width out in front
  of the chest. `front` already reached the drawer face and the code doubled it.
- Every **wardrobe door leaf**, same doubling, hanging thirty-two centimetres
  off its carcass.
- Six **sockets screwed to thin air**, because a room bound runs across
  doorways and cased openings as happily as across plaster.
- Both **bath taps' handles**, offset diagonally off two axes at once so they
  missed the tap body entirely.

The house passes clean now. Fixed viewpoints are very bad at showing that a
thing is four centimetres off the surface behind it, and that is precisely the
mistake procedural geometry makes over and over, because every position here is
arithmetic and arithmetic is off by two.


## Pass thirty-three: a title screen, and full screen

- **The title screen is the game.** No separate scene and no still image: the
  house is already loaded, already lit, and already the best thing in the build,
  so the menu is the great room seen from a corner with the enamel sign hung
  over it and a scrim behind the lettering.
- **New Game does not cut.** It flies the camera down to the fly over three and
  a half seconds — the same fly that has been sitting on the ceiling the whole
  time you were reading the menu — fades the sign out over the first half, and
  takes the mouse the moment control lands. A cut would say the menu and the
  game are two different places. They are not.
- Done as a system that runs *after* `place_the_eye` and interpolates toward
  whatever it wrote, rather than branching inside it. The chase camera is a lot
  of feel that took a long time to get right and the way not to disturb it is
  to leave it running.
- Input is gated on the dive having landed. Reading a menu should not fly a fly.
- **Full screen to play, windowed to work on.** Every capture and diagnostic
  switch implies windowed: those run dozens of times an hour on a machine
  somebody is using, and a build that seizes the display each time is a build
  nobody runs. `F11` toggles.
- `FLY_DIVE=<seconds>` starts part-way through the move so it can be captured
  at a chosen moment rather than guessed at.


## Pass thirty-four: moving the furniture yourself

Brett asked whether a mode for arranging the house by hand was plausible. It
was, and the obstacle was not the moving — it was that **nothing had identity**.
`furniture.rs` emits loose boxes; a sofa is about forty of them and no part of
the generator ever said which forty.

- **Pieces.** Every constructor now tags what it added with the index of its
  first box: unique without a counter, stable because the generator is
  deterministic, and nested calls sort themselves out for free — the inner
  constructor tags first and the outer paints over it, so a cushion belongs to
  its sofa. Anything left unclaimed becomes a piece on its own, which is right
  for a mug or a toy. **1490 boxes in 673 pieces.**
- **The fly is the cursor.** No editor camera and no orbit rig: you are still
  the fly, you fly over to the thing, and what you are pointing at is whatever
  is in front of you. Tab in and out, look to highlight, E to take and drop, Q
  and R to turn, Ctrl+S to save, Backspace to put the whole room back.
- **What comes out is a file the generator reads.** A layout worked out by hand
  survives a rebuild, which is the part that makes this worth having rather
  than a toy.
- Verified: grouping, picking, the highlight, and loading a hand-written
  arrangement (a bookshelf moved and turned on the next run). Not verified by
  capture: the grab and the save keystrokes, which a screenshot cannot press —
  they share `shift` with the load path, which is exercised.


## Pass thirty-five: arrange mode, as reported from actually using it

Brett tried it and found three things, two of which were my fault outright.

- **It fought the game for its keys.** `Q` is the first-person toggle, `R` rolls
  the camera and `E` cycles the ajar door — three of the four keys I picked were
  already bound. Take and drop is a click or `G`, turning is the arrow keys, and
  `F4` toggles the mode as well as `Tab`.
- **There was no crosshair.** "Point at the thing" is the entire interface and
  there was nothing on screen saying where the ray went. That is almost
  certainly why `E` looked broken: with nothing to aim, most looks land on a
  wall, and a wall is not a piece.
- **The flight model is wrong for this job, and that is not a bug.** It is built
  so a fly *cannot* hold a position: no lift without thrust, no reverse, a
  committed course that snaps rather than steers. All correct, all miserable
  when you are lining a sofa up with a wall. While arranging, the fly hovers —
  direct control, full stop when you let go, reverse included. The flight model
  itself is untouched.


## Pass thirty-six: the legs walk

- Six legs that hold one pose while the fly slides along the floor read as a
  model being dragged. They walk now, on an **alternating tripod** — front and
  hind on one side with the middle leg of the other, so three feet are always
  down. That is the actual gait, and it is the reason a fly can walk up a wall
  without falling off it.
- The cycle is driven by **distance covered, not time**. Walk slowly and the
  legs move slowly, by construction, so the feet never skate. A leg lifts only
  on the half of its cycle where it should be off the ground, so the three that
  are down stay down.
- `FLY_GAIT=<0..1>` poses the cycle from outside, because a screenshot cannot
  walk. Captured at three phases to confirm the tripods actually alternate.


## Pass thirty-seven: a ghost you can read

- The highlight was one translucent bounding box, which says where a thing is
  and nothing about which way it faces — useless precisely when you are turning
  it. Every solid in this house is a unit cube with a transform, so a faithful
  ghost is those transforms again a shade larger: a pool of cubes, and a piece
  borrows as many as it has boxes. A bookshelf now ghosts as uprights, boards
  and books.
- The keys line only shows the keys that do something *now*. Turning works only
  while carrying, and a line that lists every key at all times is a line nobody
  reads.


## Pass thirty-eight: the ghost moved and the furniture did not

Brett: the ghost drags and places fine, and the object never moves. Two bugs,
and the symptom named them exactly — the ghost is drawn from `Home.solids`, so
if the ghost moves then the solids are moving and it is the *entities* that are
not following.

1. **`Part` was never attached.** The component existed and nothing in the house
   had it, because the script that added the struct and the one that attached it
   were the same script and it aborted between the two. So the query in `shift`
   matched nothing, every frame, silently.
2. **The load raced the renderer.** `dress_the_set` spawns through deferred
   commands, so a `Startup` load either mutates solids the renderer has already
   read or updates a query that is still empty, and which of the two you get is
   scheduling order. It runs in `PostStartup` now, after the flush.

The second was hidden by the first: before `Part` existed, loading happened to
work anyway because the renderer read the already-mutated solids. Fixing one
made the other visible.


## Pass thirty-nine: height, and things that are one thing

Both from Brett using it.

- **Raise and lower**, on the up and down arrows. Height is on its own keys
  rather than following the fly: tying it to where you are hovering makes it
  impossible to slide something along a shelf without lifting it off. The saved
  file carries `piece x y z yaw` now — getting a mug off the floor and onto a
  shelf is half of what arranging is for.
- **Composites are one piece.** Constructors were already grouped, but plenty
  of objects are assembled from loose slabs in the room code and each slab was
  its own piece: the bath and its tap and screen, the shower, the linen press,
  the media unit and its television, the kitchen island, the fridge and its
  doors, the cooker and its whole face, both machines and their fascias. Marking
  a block is two lines — take the length before, `piece_up` after — so no
  restructuring was needed. **673 pieces down to 559**, which is 114 loose boxes
  finding the object they belong to.
- The floating law audits the *generated* house, not the arrangement, so lifting
  something onto a shelf does not trip it. That is the right way round.


## Pass forty: making the save answerable

Brett asked how to save, and the honest answer was `Ctrl+S` — followed by two
problems with it.

- **It wrote inside the application.** Beside the executable was the first
  answer and it is wrong for anything installed: the launcher replaces the whole
  bundle on update, so every layout anybody had worked out would go with it, and
  the bundle may not even be writable. Saves go to
  `~/.fly-on-the-wall/arrangement.txt` now. A build can still ship one of its
  own beside the executable, and that is read as a fallback.
- **Nothing said it had worked.** There is no console when the game starts from
  the launcher, so a save that only logged was a save nobody could tell had
  happened — and "did that work?" is the single question a save has to answer.
  It now says so on screen, with the count and the path, for five seconds.
  Backspace says so too.


## Pass forty-one: the father

Voxel language, as the design has said from the beginning, but jointed. A
Minecraft body is six boxes and its arm swings from the shoulder in one piece,
which is why it reads as a puppet: nothing between shoulder and hand ever
changes shape. This one has shoulder, elbow, wrist, hip, knee and ankle, so a
pose is a set of joint angles and an animation system has something to drive.

Three things do most of the work of looking better than the reference, and none
of them costs the blocky read:

- **Taper.** Every limb segment is five boxes, each a shade narrower. A thigh
  thick at the hip and thin at the knee is nearly all of what makes a leg read
  as a leg. Three segments was the first try and it was worse than none: three
  ledges down a forearm read as three separate blocks. Finer steps with an
  overlap put the change where the eye takes it as shape.
- **Joints that exist.** A block at each elbow and knee, so a bend has
  something in the corner instead of two prisms passing through each other.
- **Proportion.** Seven and a half heads, not four. The reference is a
  caricature and standing one in a room built to centimetres would make the
  room look wrong rather than the man.

And **eyes**. A head with a brow and a nose and no eyes is a mannequin; two
boxes two centimetres across do more for this model than anything else on it.

He is scenery: entities and transforms, no collision and no behaviour. The
family simulation is a long way off and this is the body it will be given.
`FLY_FOLK=<deg>` stands in front of a person and looks at them, because the
room views frame rooms and neither they nor the fly's inspect view can show
whether a knee bends the right way.


## Pass forty-two: his feet were on backwards

Brett, immediately. The body faces −z — nose, brow and fringe are all on that
side — and the shoe's long axis was laid on +z, so he stood in his own living
room with his feet the wrong way round. Invisible in the numbers, unmissable to
anybody who looks at him.

Worth recording the second half of it: the viewpoint I had just added to look
at people **could not show his feet**. It framed the chest at three metres and
cut off below the knee, which is a poor showing for a camera whose only job is
looking at a body — and it is why I did not catch this myself. It frames the
whole person now.


## Pass forty-three: a made model in a generated house

Brett can author models now, and pointed out that the law says *I* do not build
assets in Opificium — not that he cannot hand me one. Fair reading. The couch is
his.

**The seam is deliberately small.** A `Solid` can carry `model: Some(path)` and
`unseen: true`. The generated piece is still built: it is what the fly lands on,
what arrange mode picks up, and what the house falls back to if an asset goes
missing. The model is only what gets *drawn*. Nothing downstream — collision,
pieces, the arrangement file, the floating law — has to know which kind of
furniture it is looking at.

- Opificium exports with an `opificium-fit` node normalising to real-world
  metres, so the only conversion needed was centimetres: scale 100.
- The couch came in at 216 x 98 x 100 cm, sat on the floor, right way round,
  first try.
- The procedural cushions and blanket had to go: they were positioned against
  geometry that is no longer drawn, and floated beside the model.
- Cost: 96 meshes (from 15) and 10,695 triangles for the one model. Frame rate
  unchanged.

The style holds together better than expected — shared lighting and a muted
palette do most of it — but the model is plainly the best thing in the room now,
which raises the question of what else gets replaced and in what order.


## Pass forty-four: collision from the mesh

Brett: give the model the collision and delete the old couch. Right — keeping an
invisible generated sofa in step with a model is exactly the maintenance the
seam was supposed to avoid, and it does not scale to a house full of models.

A model arrives as ten thousand triangles and this game's collision is oriented
boxes, which the entire flight model rests on and which is not worth replacing
to seat one couch. So the triangles become boxes: the mesh is voxelised on a
seven-centimetre grid and the occupied cells are greedily merged back into as
few boxes as will cover them. **10,695 triangles to 140 boxes.**

- Occupancy by barycentric sampling of each triangle rather than exact
  triangle-box overlap: at seven centimetres it misses nothing that matters and
  is a tenth of the code.
- The boxes join the model's piece, so arrange mode still moves the couch.
- The generated sofa is *gone*, not hidden. `use_model` truncates whatever was
  built for the piece.
- `FLY_HULL=1` draws the result. Collision that cannot be seen is collision
  nobody can check, and a hull derived from a mesh is the sort of thing that
  looks right in a log line and is wrong in the room. It follows the arms, the
  back, the cushions and the base.

Any model dropped in from here gets its own collision with no hand-authoring.


## Pass forty-five: the collision was way off, and it was

Brett said so and he is right. The voxel hull was a seven-centimetre shell round
the couch — proud of the arms, filling the dip in the seat, bridging the gap
between the cushions. My own comment in that file argued that coarse was fine
because "the upholstery does not need to be accurate to a quarter of a
centimetre", which is exactly the wrong reasoning for this game: seven
centimetres is fourteen body lengths to the thing landing on it.

**Made models now collide against their own triangles.** Möller-Trumbore for
rays, Ericson's closest-point for contact, both bucketed on a twelve-centimetre
grid that is only a filing system and has nothing to do with accuracy. The mesh
you can see is the surface you land on. `FLY_HULL=1` fires a grid of downward
probes and draws a speck where each one stops: they sit on the piping and follow
the roll of the arm.

**What I did not do, and why.** The advice Brett brought suggested Rapier or
Avian with trimesh colliders, CCD, and surface-constrained crawling. Three of
those are already true here and one is a bad trade:

- The world is in **centimetres**, not metres. A fly's body radius is 0.26
  units. The floating-point tolerance worry does not apply.
- **Swept collision already exists** — the flight model raycasts from the
  previous position, because at 200 cm/s the fly covers 3.1 cm a tick and the
  thinnest solid is a 2 cm pane. Tunnelling was solved months ago.
- **Surface-constrained crawling already exists** — `walk_about` re-seats onto
  whatever is underfoot every step and orients to its normal, which is why it
  goes floor to wall to ceiling. It now does that on the couch's actual
  triangles for free, because it re-seats with a raycast.
- The generated house's **boxes are exact**: a wall *is* a box. Nothing there is
  being approximated, so a physics engine would replace a working, tuned system
  and the approved flight model to fix a problem that only ever existed for
  imported meshes.

One thing genuinely worth watching: at a seam between two triangles the contact
normal is whichever face wins the closest-point test, which can flip across an
edge. That is the "ghost collision" the advice mentions. It has not bitten yet
and the fix if it does is to blend normals near an edge, not to add an engine.

Cost: 103 fps uncapped, unchanged.


## Pass forty-six: the Esc menu

Resume, Title Screen, Exit Game, and nothing else. A pause menu is the answer to
"how do I stop", and every extra line on it is a line between somebody and the
door.

- **Pausing actually stops the fly.** Input and the fixed-step simulation are
  both gated on it, so a fly left hovering does not sink into the floor while
  somebody reads the menu.
- Escape used to release the cursor; the menu does that itself now.
- **Title Screen puts the fly back on the ceiling.** A "New Game" that drops you
  wherever you happened to be standing is not a new game, and the dive is
  written to arrive at the spawn. `hatch` split into `at_spawn` for it.
- `raise_the_sign` runs every frame now instead of once at startup, guarded on
  there not already being a sign, because the menu can send the game back to the
  title and the sign has to be there when it arrives.

**A bug found on the way, older than any of this.** The dive's fade took
`Query<&mut TextColor>` — *every* piece of text in the game, not just the sign's
— and ends at zero alpha without putting it back. So after one dive the F3
readout and the arrange HUD were invisible for the rest of the session. Nobody
had seen it because every capture switch skips the title, so no screenshot I
have ever taken went through a dive. Scoped to a `TitleFade` marker.

`FLY_PAUSE=1` opens the menu on the first frame, because a keypress cannot be
captured and every other screen in this game can be.


## Pass forty-seven: the armchair

Second of Brett's models, into the reading corner, and the pipeline held: two
lines in `living`, collision from its own 11,000 triangles, no proxy authored.

Two things worth keeping:

- **The first export had no `opificium-fit` node** and came in centred on its
  own origin, so half the chair was under the floor. Brett fixed it at the
  source rather than in the game, which is the right end of the pipe: the fit
  node carries both the real-world size and the base-at-zero, and without it
  the game is guessing at both.
- **The made chair is a hundred and sixteen centimetres wide where the
  generated one was ninety-two**, and the father was standing in it. Swapping a
  generated piece for a model changes its footprint, so anything positioned
  near it by hand has to be looked at again. That will keep happening as models
  land.


## Pass forty-eight: the walk cycle

Brett asked for the legs to animate while walking. They already had a tripod
gait written for them — and it had never once played.

**`walk_about` zeroes the velocity every tick.** A perched fly is *placed*, not
integrated, so `vel.length() > 0.35` was false on every frame a fly has ever
walked. The animation was correct-looking code that no capture and no play
session had ever run. The gait is driven by the body's own displacement now,
which is also what makes it immune to how walking is implemented.

What the legs are now:

- **Femur, knee and tibia**, not one stick. The knee is the whole insect
  silhouette — femur up and out from the thorax, tibia back down to the foot —
  and a single segment cannot do it, because its foot can only travel on an arc
  about the anchor.
- **Posed by the foot, not the joints.** A target is worked out in the body's
  frame and the two bones are solved to reach it. That is the only way a
  planted foot stays planted: the body moves, the target does not, the knee
  takes up the difference.
- **A real duty cycle.** Sixty-two per cent stance, and the cycle carries the
  body exactly one stance's worth of foot travel, so a foot on the ground does
  not slide — by construction rather than by tuning. There are moments with all
  six feet down, which is what a walking insect has.
- Swing is eased at both ends so a foot sets down rather than arriving, and the
  stride eases in and out so nothing freezes mid-swing when you let go.
- Speed for that ease comes from the last *tick*, not the frame: drawing runs
  faster than the fixed sixty-four hertz, so a per-frame speed flickers between
  walking pace and zero.

**Two tool fixes it forced.** `FLY_INSPECT` looked *down* on the fly from above,
which is the one angle that cannot answer "is that foot on the floor" — it is
nearly level now. And it had no light on it: the fly stands three metres from
the nearest window and the model was too dark to judge, which makes an
inspection view that cannot inspect.


## Pass forty-nine: moving and resizing made models

Two asks: move imported models in arrange mode, and resize with the wheel.

**Models could not be moved at all, and the reason was two bugs deep.**
`bounds` measures a piece from its solids, and a model's solid is a
four-centimetre stub carrying an asset path — so it skipped them, found nothing,
returned `None`, and `shift` gave up before doing anything. A model solid now
contributes its *position* and not its size, and `bounds` counts the hull, so a
couch is measured by the couch.

**And the collision would have stayed behind.** Hull triangles are held in world
space — that is what makes the queries cheap — so a couch dragged across the
room left its collision standing where it was: invisible in a screenshot and
unmissable the moment you fly into it. `Hull::place` moves, turns and resizes
them and refiles the grid.

**The wheel resizes**, half to double, about the middle of the footprint at
floor level — scaling about the centre grows a chair down through the floor.
Solids scale their halves, models scale their transform, hulls scale their
triangles, and the size is saved so it survives a reload. The scale column is
last, so a file written before it existed still loads.

Two tests, because this is geometry and the failure is silent: a hull must
arrive where its model went, and shrink where its model shrank. Both would have
failed this morning.

## Deferred

- Family simulation, needs, danger, death, objectives, progression, and HUD are
  outside the active house-and-lighting goal.
- Flight and camera redesign are deferred unless house validation reveals a
  specific regression that blocks traversal or inspection.
- Hinged door leaves. Openings are honest holes for now; a swinging door is its
  own piece of work and the greybox still has the only one.

## Next

- Then: fly-scale routes, undersides and landing surfaces; restrained clutter.

## The father becomes a made model

Brett supplied `assets/characters/DadRigged.glb` — a Tripo-rigged humanoid, one
mesh, 4776 triangles, one material with base colour, normal and
metallic-roughness maps, and a 41 bone skeleton with the standard names
(`L_Upperarm`, `R_Calf`, `Head`). No animation clips.

He replaced the hand-built body. That body had two lives in `folk.rs`: boxes
with real elbows and knees, then lathed surfaces of revolution with leaning
rings. The lathe version was the better engineering and still losing — a face is
not a stack of ellipses, and each pass fixed one feature and exposed the next.
Both are gone; git has them.

Kept from the hand-built work: the `Person` marker, the height on `Stature`, and
collision taken from his own triangles.

### Three real bugs found along the way

**Inverted winding in the lathe.** Every surface of revolution was wound
backwards, so each shape rendered as its own far wall. A closed convex shape
looks nearly right that way — the silhouette is identical — which is why it
survived several passes. What gave it away: shading went flat (normals pointed
inward, so everything was lit from behind) and nested shapes broke, because the
depths were inverted. A hair shell wrapped round a skull drew *behind* it, so
the man was bald with a coloured rim. Diagnosed by rendering the hair bright red
and then oversizing it: the head poked through a dome that entirely enclosed it.

**Collision built before transforms propagated.** `pose_him` writes local bone
rotations in `Update`; the world positions those imply are computed in
`PostUpdate`. Building the hull in `Update` gave a man 117 cm across the
shoulders — the bind T-pose — while the screen showed his arms down. Moved to
`PostUpdate.after(TransformSystems::Propagate)`: 179 cm tall, 59 cm across.

**Skinned meshes never leave the bind pose in memory.** Skinning runs on the
GPU, so reading `ATTRIBUTE_POSITION` the way the couch is read gives the T-pose
whatever is on screen. `make_him_solid` now skins on the CPU once — each vertex
moved by `joint_world * inverse_bindpose`, weighted — so the surface the fly
lands on is the surface anybody can see. Once only; a walking body would need a
cheaper answer than refiling four thousand triangles.

### New tool: `FLY_STUDIO=<deg>[:head]`

A turntable. Hides everything that is not a person, lights it three-point on a
flat ground, and orbits **relative to the person's own facing** so nought
degrees is dead ahead of him rather than of the world.

Built because the house kept preventing the work. `FLY_FOLK` stands 3.5 m out
and at half the compass that is inside a wall — one capture came back as a sheet
of magnolia. Most of what was wrong with the hand-built father (head two thirds
the height it should be, hair reading as sunglasses) was in plain sight and
simply could not be seen from in there.

### Decisions from Brett

- The blank face is deliberate style, not a defect. Characters never speak
  intelligibly — the fly cannot understand them — so expression comes from body
  language. Do not add facial features.
- The cast is the fly plus four people and the occasional visitor. That ratio is
  the argument for how much care goes into bodies.
- Clothing should be **part swapping, not layering**: a garment replaces the
  drawn version of the segments it covers. No underwear body, no poke-through,
  and collision derived from what is drawn means a bulky coat gets bulky
  collision for free.

### Poses and the idle

Both are authored in `folk.rs` as tables of numbers, not clips. The file ships
no animation, and a table can be tuned by editing a number — the same argument
this game makes everywhere else for building things in code.

`AT_EASE` is thirteen bone rotations composed **onto** the bind rotation. A bind
pose already carries each bone's own orientation and replacing it snaps every
limb onto the armature's axes. The first attempt had the sign wrong and he stood
with both arms straight up.

`IDLE` is ten bones on sine waves over the resting pose, driven by `breathe` the
way the fly's wings are: breathing at fourteen a minute with the neck taking it
back out so his head does not nod along with his lungs, a weight shift at four a
minute, and the head drifting slower still. Rates are near-coprime so it does
not visibly loop. Nothing here is meant to be noticed — stillness is the one
property no living thing has, and its absence is what reads.

The arm amplitudes are smaller than they want to be, and that is a collision
constraint, not a taste one — see below.

### `FLY_HULL=1` now draws every hull

The probe grid was hardcoded to the couch's bounds inside `fit_collision`. It is
`made::probe` now, sized from whatever hull it is given, and folk call it too.
Probes are marked so the turntable does not hide them. Confirmed the father's
collision: specks land on his shoulders, chest, forearm and shoes.

### Next: collision that follows the bones

His hull is built once, from the pose he settles into. That bounds how much the
idle may move: a hundredth of a radian at the shoulder is most of a centimetre
at the hand, which is two body lengths to the fly.

Worse, `Perch` is stored relative to a *solid*, and his solid never moves — so a
fly landed on a sleeve stays put while the sleeve does not. Rebuilding the whole
trimesh is the wrong answer (a few milliseconds, and it does not fix the perch).

The answer is per-bone collision: a capsule or a small hull per bone, posed with
it, and perches stored relative to the bone rather than the body. That is what a
walk cycle needs before it can exist.

## Sitting him down, and finding the seat rather than remembering it

A man standing motionless in the centre of his own living room was the oddest
thing left in the house. He sits on the sofa now, watching the television.

`Posture` is a value rather than a constant: the bones that differ from the bind
pose, and a rough starting height. There are four people coming and the
interesting thing any of them does is sit down.

Getting the seated pose right took several rounds, and two of the mistakes are
worth remembering:

- **Flexion is about x, not z.** Written about z first, which is abduction: he
  sat with his legs straight out sideways in the splits.
- **A model's origin is only between its feet while it is standing.** Fold the
  legs and the feet end up half a metre in front of the origin and well above
  it. Authoring that offset per pose is a number nobody can eyeball — the first
  seated father had his shoes fifteen centimetres into the floorboards, with one
  visible under the sofa. So the drop is *measured*: `make_him_solid` reads the
  bottom of the hull it just built and shifts the whole body by it. Exact in one
  correction; the retry only exists so transforms can propagate before the hull
  is rebuilt. Every future posture gets this for free.

### Brett resized the sofa

Mid-session, in arrange mode, and saved it. That is exactly the event that
should break nothing, and it left the father sitting in the air beside a smaller
sofa — he had been placed at coordinates copied out of one afternoon's log.

So `made::seat` finds a seat from a model's own collision. Probes are dropped
over its footprint; a cushion is the low plateau, a back is the high one, and
the direction from one to the other is the way the piece faces. Thresholds are
fractions of the piece's own height, so scaling it changes nothing.

The first version took "anything at roughly seat height" as cushion and put him
a foot inside his own sofa. A ray dropped just off the back edge grazes the
sloping outside and lands anywhere between the top and the floor, so a band
alone collects a scatter of hits behind the seat. A cushion is a *plateau*: bin
the heights, take the fullest bin, keep only what is level with it. Graze hits
spread across every bin; a cushion fills one.

### Diagnostics added

`made` now reports, for every model it collides: size, position, the height of
the top surface at its centre, and a surface profile across x. That profile is
what identified which way the sofa faced — reading it off the code that placed
the furniture does not work, because the generated sofa was thrown away and the
model brought its own orientation.

### Answered for Brett

A single glb can hold many animations — glTF has a named `animations` array and
Bevy loads them all. One file is better than several: the mesh and textures are
stored once. Separate files also work, because Bevy matches clips to bones by
name path, but each export usually carries the whole mesh again. What matters is
that every clip comes off the same skeleton with the same bone names.

## Animation playback, ahead of there being any

Brett is rigging animations in 3daistudio, so the path is in place before the
file is. `play_what_he_has` looks for clips on the model a person came out of;
if it finds any, one plays and the hand-built pose and idle stand down. Nothing
hand-written should be fighting an animator for the same bones.

The clip is chosen by name where the name says what it is — a posture asks for
`sit`, anything called `idle` will do otherwise — falling back to the first clip
in the file. The current model has none, so the hand pose still runs; that is
the fallback working, not a failure.

Two things this depends on and neither is guaranteed by an exporter: every clip
must come off the same skeleton with the same bone names, and Bevy's loader must
put an `AnimationPlayer` on the animated root, which is how the system knows the
skeleton has arrived at all.

Still open: an animated body invalidates the once-built collision hull. A seated
idle drifts by millimetres and does not matter. A walk will, and the answer is
per-bone collision with perches stored against bones — see above.

## One animation per export is fine

3daistudio writes a single animation per file. Nothing needs merging: Bevy
matches a clip to a skeleton by the **name path** of each bone, so a walk
exported from this rig plays on the body already standing in the room, and the
duplicate mesh inside the walk file is simply never used. Combining the files
would save disk and change nothing else.

`folk::movements` scans the characters folder rather than listing files, so a
new movement is a file drop and not a code change. The name comes from what
differs: `DadRigged` and `DadWalk` share `Dad`, so the second is `walk`. That
rule is a pure function with a test, because the files that will exercise it do
not exist yet.

All the clips go into one `AnimationGraph`, and the node index of each is kept
on the person as `Repertoire`. Changing what somebody is doing is then choosing
a node, not loading a file — which is what a state machine will want.

Requirements on the exports, none of which an exporter guarantees: same
skeleton, same bone names, and no root motion baked in. The game moves a body;
the clip should not.

## He walks about on his own

Brett delivered `assets/characters/dad/dad-idle.glb` and `dad-walking.glb`: same
41-bone skeleton, clips named `preset:biped:idle` and `preset:biped:walk`, and
the bind pose is now arms-down rather than a T.

### The travel is taken, not cancelled

The walk has its motion baked in: the hip ramps 1.574 model units over 2.38 s,
which on a 178 cm body is about 1.18 m/s. Left alone it carries the *drawn* body
across the room while the entity stands still — the first capture had him walk
into the camera.

Cancelling it and moving him at a speed chosen here would be wrong twice: it
throws away the animator's timing, and any disagreement between the two speeds
shows as feet skating. So the travel is taken. Each frame the hip's sideways
motion is read, applied to the body's place in the room, and put back. He moves
exactly as fast as he is animated to move, because it is the same motion.

Only while walking. A standing idle also swings the hips, but a standing body's
feet are planted and its hips move *against* them; carrying that into the room
would slide him sideways while stood still.

### Two bugs

**He stood still for twenty seconds.** The clearance test fires rays from the
walker's own position, and a person's own collision is in the same list as the
walls — so every direction reported blocked. Rays start 36 cm clear now.

**Brett found this one: he walked three feet off the floor and stood on it
perfectly.** Putting the hip bone back read `rest + up * height`, which *adds*
the bind height to the animated height rather than replacing it. The hip's bind
height is half the body — 0.51 units, 91 cm, almost exactly three feet. It only
showed while walking because the idle never runs that code. `rest` is stored
flattened across the up axis now.

### Collision follows him

`Hull` carries a rigid `shift`, so a body that moves does not refile four
thousand triangles into a grid every frame — queries move into the filed frame
instead, two matrix multiplies. `Filed` records where a person's hull was built;
`carry_the_collision` sets the shift from where they have got to, and moves the
anchor solid so perches stay attached. The limbs are still the pose the hull was
built in; the *body* is where you can see it, which is the difference between
landing on a man and landing on a ghost.

### `FLY_STUDIO=<deg>[:head][:keep]`

`keep` leaves the house standing and brings the camera in close with a wider
lens. The framing is the useful half of the turntable when the question is about
a body *in* a room — where he has walked to, whether his feet are on the
floorboards — and the room viewpoints cannot answer it, because at half the
compass they stand inside a wall.

### Next

- Doorways. He only picks destinations in the room he is already in, because a
  straight line is the only path he can test. Walking to another room needs the
  openings as waypoints.
- The hull is still the pose it was built in. A perch on a swinging arm is
  wrong by up to a hand's width.

## Two bugs Brett found

### The twisted elbow

Both clips put the forearms in a bad roll: palms turned to face forward and a
pinch at each elbow. Measured against this rig's own bind pose, the deviation is
**69 degrees on the right and 159 on the left**, both about the forearm's own
length axis. Asymmetric by ninety degrees is not something an animator does on
purpose — it is the preset being retargeted from another skeleton.

Proving it was the clip and not the mesh needed a way to see the model with
nothing playing, because a clip drives every bone it has a channel for and hides
whatever is underneath. `FLY_MOVE=none` does that, and the bind pose turned out
to be perfect: arms hanging, palms to the thighs, no pinch.

The fix takes out the roll and nothing else. A rotation splits cleanly into a
swing and a twist about a chosen axis; dropping the twist leaves every bend the
animator wrote and removes only the spin the retarget added. The axis is read
from the rig rather than assumed — a bone's own length is where its longest
child sits, which is `(0, 0.14, 0)` for a forearm here.

### Saving the furniture threw the furniture away

Brett moved the sofa, saved, played, opened arrange mode again, pressed the same
keys, and got the generator's sofa back.

The save was written from the pieces moved *in this session*. Loading a file and
then saving without touching anything therefore wrote an empty file — and his
was exactly that: a header and nothing else. The second save is what destroyed
the first one.

`load_arrangement` now records where each piece stood *before* the file was
applied, so a saved offset is always measured from where the generator put the
thing rather than from where this session found it. Saves are cumulative.

`FLY_SAVE=1` writes the arrangement on startup and compares it with what was
read, so the round trip can be exercised without a keyboard. It reports "reads
back exactly as written", which the old code could not have done.

## Aiming stopped feeling floaty

Brett: "the mouse aim when flying feels floaty and should be mouse perfect,
flies change direction immediately".

The aim itself was already exact — `look_around` writes yaw and pitch straight
from the mouse with no smoothing, and the chase camera does not smooth its look
target either. The float was downstream, in two places:

**The dead zone.** Thrust goes along a *committed course* that only snaps to the
aim once the error passes `SACCADE_ARC`, which was twelve degrees. The crosshair
is exactly where you point and the fly keeps going where it was going, and that
gap is the whole complaint. Four degrees now, which resolves in a single tick at
1600°/s, so the course is never more than a flick behind the crosshair and still
moves in steps rather than a curve.

**Momentum left behind.** A saccade turned the course and nothing else, so the
new direction only took effect as fast as thrust could build along it and drag
could bleed the old one off. That is an aircraft banking. A fly redirects its
thrust and keeps its speed. The same rotation that turns the course now turns
the velocity with it, at `CARRY`, which is one.

The polyline character is untouched — the course still holds and snaps, and
setting `SACCADE_ARC` to zero still gives the honest continuous-curve
comparison. Feel is Brett's call; both constants are named and adjacent.

## `FLY_DIVE` was capturing a dive with no title in it

`raise_the_sign` only spawned while `Stage::Title`, and `FLY_DIVE` starts part
way through `Stage::Diving` — so every capture of the dive came back with no
title screen in it at all. Which meant a fade fix looked verified when the thing
being checked had merely gone missing. It spawns during the dive too now.

The fade itself was re-specifying each colour as a literal — the border's fade
carried its own copy of the title's cream — and had simply forgotten
`BackgroundColor`, so the New Game button left a dark rectangle over the kitchen
until the dive ended. It snapshots the real colours on the first frame of the
dive and multiplies, so nothing can be forgotten or contradicted again. The
scrim fades with it now too.

## The dead zone had to go entirely

Brett, after the first pass: "now when you mouse look to turn the fly azimuth
clicks through positions instead of smoothly aiming".

That is the quantisation, felt rather than seen. The committed course holds,
snaps, holds, snaps — and since the velocity now comes round with each snap, the
steps are felt in where the fly actually goes. Twelve degrees read as float; four
read as ratcheting. A dead zone cannot be neither: either the control lags the
hand or the hand feels the steps.

So `SACCADE_ARC` is zero. The quantisation comes out of the control and stays in
the body — `SACCADE` still snaps the drawn heading, which is the half anybody
can see, and the camera has always looked along the aim rather than the body.

**The cost, stated plainly:** the polyline path is gone. The course follows the
aim continuously, so a turn traces a curve rather than a sequence of straight
segments, and because the body's target is now continuous its 14-degree snap
only trips on fast flicks. The insect character is thinner than it was. That is
the trade Brett asked for and the constant to raise if he wants it back.

`CARRY` at one also means velocity direction is locked to the aim: the fly can
no longer coast in a direction it is not pointing. Immediate, as asked, but the
third failure mode after float and ratcheting is weightlessness, and 0.6–0.8 is
the dial for that.

## The floating couch: my fix, my bug

Brett: "not sure why but the couch is floating now. I could fix it, but I
thought it not saving correctly might be a bug". It was.

Making saves cumulative, I seeded the move record at load time by *measuring*
where each piece stood. `bounds` warns about this in its own comment: at load
time a made model's hull does not exist yet, so it reports the four-centimetre
stub standing at floor level rather than the couch. The save then measured the
couch's real centre against a floor-level point and wrote the difference as
though somebody had lifted it — and it compounded on every cycle. The sofa's
saved height went `-3.89` → `+42` → `+88.33`.

The record holds **totals** now — how far a piece has been shifted, turned and
resized from where the generator put it — accumulated as it is dragged and
copied straight out of the file on load. The file holds offsets, so holding
offsets in memory makes the round trip a copy rather than an arithmetic
reconstruction through a measurement that cannot be trusted at that moment.

Repaired Brett's file: the two made pieces had their `y` zeroed, because under
the corrected scheme a piece that was scaled but never lifted needs no
translation at all — `shift` already resizes about floor level so a shrinking
couch stays on the floor. The old `-3.89` was itself an artefact of measuring
centres. The unscaled pieces were unaffected and were left alone, including a
genuine 54 cm lift on 3731. A copy of the file as found is in the scratchpad.

## The stoop

Brett: "the dad looks like he has scoliosis, lol". Head hanging, spine curved.

Same fault as the forearm roll, and the same lesson about viewpoints: the idle
was checked from the front, where a forward lean is very nearly invisible.
Twenty degrees of stoop reads as nothing head-on and as a hunchback from the
side. The turntable's angle is now part of what gets checked, not an
afterthought.

Measured against the rig's bind pose, `NeckTwist01` sits 12.7 degrees forward
through the whole idle and the three spine bones add three and a half each —
about twenty-three degrees, and thirty-four in the walk.

What makes it correctable is that it barely moves. The neck's deviation ranges
over four degrees across fifteen seconds; the spine's over one. It is an offset,
not a performance. So a constant is subtracted, composed onto the deviation
rather than onto the bone, which takes out the stoop and leaves every bit of the
animation intact.

Deliberately *not* a pull back toward the bind pose. That would work here and
would flatten a sitting clip into standing the moment one arrives.

Under-corrected on purpose: the walk legitimately leans further than the idle,
so removing roughly the idle's offset leaves a walker with the forward lean a
walker should have.

Three retarget faults found in these two files now — forearm roll, spine stoop,
and the baked travel. All three were constants sitting under the animation, and
all three were measured against the rig's own bind pose rather than guessed at.
`FLY_MOVE=none` is the tool that makes that comparison possible.

## The launcher build was broken and the repository build was not

Brett booted v0.3.0 from the launcher and found the father standing with his
arms crossed over the wrong shoulders, not walking. From `FotW.command` it was
perfect.

`movements()` scanned `assets`, `../assets` and `../../assets` **relative to the
process's working directory**. That is the repository root when the game is
started by hand and is not the app's folder when a launcher starts it. So the
shipped build found no movement files, and then:

- no clips → `play_what_he_has` fell through to the hand-written pose, whose
  bone angles were authored first for the hand-built body and then for a rig
  whose bind pose was a T. Against the arms-down rig that replaced both, they
  fold his arms across his chest.
- no `Repertoire` → `find_the_hip` never ran → no `Doing` → no walking.

Two fixes, and the second matters more than the first.

**The path.** `FileAssetReader::get_base_path()` is what the asset server itself
uses: the manifest directory under `cargo run`, the executable's own folder
otherwise. Bevy found the models perfectly the whole time; only this hand-rolled
scan did not.

**The fallback.** The hand-built pose tables, the idle sine waves and
`pose_him`/`breathe` are gone. A fallback that is worse than doing nothing is
not a fallback: a rigged model arrives in a pose its author chose, and standing
still in it is the right answer when there is nothing to play. Had that been
true already, the path bug would have shipped as "he doesn't walk" rather than
as a man with his arms tied.

**The lesson.** The only build ever tested was the one that could not exhibit
the bug. Verifying this meant packaging the `.app` and running it from `/tmp`
with `CARGO_MANIFEST_DIR` unset — which is now the check for anything that
touches asset paths, and is two commands.

## A rigged fly

`assets/characters/fly/fly-walk.glb` — 32 bones, one clip, `preset:hexapod:walk`.
It is a far better animal than the boxes in `body.rs`: chitin, compound eyes,
translucent wings, six jointed legs.

`FLY_MODEL` now picks between three bodies — `built`, `glb` (the early Tripo
model whose rig was unusable) and the rigged one, which is the default.
`FLY_MODEL=built` goes back.

**Facing was measured, not guessed.** Rendering the built fly and the model from
the same camera angle settles it in one comparison: the built one showed its
back, the model showed a profile, so the model's nose is its own +X and it wants
a quarter turn. Reading a facing off a filename is how the last model ended up
with its feet on backwards.

**The clip runs at the fly's ground speed**, not on a loop. `speed = ground /
PACE`, zero in the air, so the feet keep up with the floorboards instead of
skating and it stops dead when the fly does. The same argument as the father's
walk.

### What it costs

The wingbeat. `work_the_wings` drives parts this file builds, and the model
brings one clip and it is a walk — so a fly in the air holds its wings still.
That is the one place the boxes are still ahead, and it matters, because the air
is where the game is played.

The fix is to drive the model's own wing bones from the same effort signal, and
the bones are there: 32 of them, mostly named `bone_N`, so they need identifying
from the geometry rather than the names.

## Reading the fly's rig instead of trusting its clip

The rigged fly walked like it was being thrown about. Weighing every vertex
against every bone says why, and the same weighing says what *is* usable.

**The wings are rigged properly.** `bone_12` and `bone_14` drive one hundred per
cent of the wing geometry, fifty–fifty, one bone per wing: thin membranes high
on the body at y≈0.70, symmetric at z≈∓0.31. Nothing else in the skeleton looks
like that. They are driven from code now.

**The legs are not.** `tripo::0_Left_Limb_6` alone carries 882 units of weight
spread across ninety-six per cent of the model's width — the auto-rig hung
several legs off one bone. There is no per-leg control to run a tripod gait
with, and `preset:hexapod:walk` swinging that bone through ninety degrees is
what threw the animal around. The clip is off unless `FLY_WALK=1`.

This is a different kind of fault from the father's. His were *constants* —
a forearm roll, a spine stoop — sitting under otherwise good animation, and
subtracting a constant fixed them. Here the preset and the rig disagree about
what the bones *are*, and no correction fixes that. The bind pose is excellent,
so standing in it beats thrashing.

**The wingbeat is a smear, not a flap.** A housefly beats about two hundred
times a second and a screen draws sixty, so an honest flap aliases into a slow
wrong-looking flutter. The built wings answered that by widening into the arc
they sweep; the model's do the same — folded along the back at rest, swept out
and widened as effort rises. `FLY_BEAT=<0..1>` forces it, because a capture
cannot hold a key.

Method worth keeping: **bones were identified by what they hold, not what they
are called.** This rig names almost everything `bone_N`. Weighted vertex
centroids told me which two were wings, which one was the body, and that the
legs were unusable — in one pass, before writing any code against them.

## Fixing the fly's rig rather than working around it

Brett, twice: can you fix the rig? Yes — by re-skinning it, which is a
different act from every correction made so far and worth being clear about.

The father's faults were **constants** sitting under good animation: a forearm
roll, a spine stoop, a baked travel. Subtracting a constant fixed each one,
because the skeleton and the mesh agreed about what a bone *was* and only the
retarget was wrong.

The fly's rig does not agree with its own mesh. Clustering the six legs
geometrically and asking which bone owns each:

| leg | owner |
|---|---|
| front left | **`bone_7` 59%** — the *body* |
| front right | `tripo::0_Left_Limb_6` 99% |
| middle left | `bone_25` 46%, `bone_15` 18%, `bone_24` 17% |
| middle right | `bone_31` 47%, `0_Left_Limb_6` 21%, `bone_21` 13% |
| rear right | `bone_30` 63% |
| rear left | `bone_27` 63% |

Only the rear pair is drivable. Any rotation that moved the front-left leg moved
the body with it, and `0_Left_Limb_6` holds parts of three legs at once — which
is exactly what `preset:hexapod:walk` looked like when it played.

No runtime correction fixes that, so `tools/rig-the-legs.py` re-skins it: the
six legs are found by clustering everything below the body mass, each is bound
to a new bone planted where that leg meets the thorax, and weight feathers in
over the first centimetre so the joint bends instead of tearing. Six nodes and
six inverse bind matrices are appended; the mesh, the materials, the wing bones
and the existing skeleton are untouched. The original file is left alone and
`assets/characters/fly/fly-legs.glb` is written beside it.

Each leg gets **two** bones, a thigh and a shin, because one is a stick: a leg
that swings in one piece reads as a twitch rather than a step. Weight hands over
from thigh to shin *across* the knee rather than at it, so the mesh bends
instead of creasing.

`walk_the_model_legs` then runs an alternating tripod off them — phase advanced
by distance travelled, not by time, the same rule the built legs and the
father's walk follow. The thigh sweeps and lifts; the shin folds through the
return and straightens to plant.

**The bug worth remembering:** the tool built its parent map before appending
the new nodes, so `world()` walked a tree that did not contain them. A shin's
inverse bind matrix came out without its own thigh in it and the legs splayed
flat. The one-bone version had the same fault and looked fine, because the only
thing missing from it was the skeleton root's own six millimetres — a bug that
is invisible until the day it is not.

Re-run the tool if Brett re-exports the fly.

### Two process notes

**Bones were identified by what they hold, not what they are called.** This rig
names almost everything `bone_N`. Weighted vertex clustering answered every
question about it — which bones were wings, which was the body, which legs were
drivable — before a line of code was written against it.

**`cargo fmt` reflows `add_systems` tuples, and a string-replace against the
unformatted text silently misses.** That cost two rounds of "the system does not
run" on work that was already correct. The compiler said so both times —
`function is never used` — and it was faster to read that warning than to
re-derive the bug.

## The legs were animating all along — too fast to see

Reported twice as "the legs don't animate when it walks", and both times a
forced capture showed them stepping perfectly. That gap was the whole clue and I
read it as a scale problem instead of a *rate* problem.

`fly::WALK` is six centimetres a second. A fly's real stride is about a third of
its body length — under two millimetres on this body — so a physically honest
`STRIDE` of 0.18 cm puts the gait at **thirty-three cycles a second**. Nothing
can draw that. The legs ran the whole time and aliased into looking still.

This is the same trap the wingbeat has a paragraph about in the same file: two
hundred beats a second against sixty frames aliases into a slow wrong-looking
flutter, which is *why* the wings smear instead of flapping. Having written that
reasoning down, I then walked straight into it with the legs.

`STRIDE` is a centimetre now — six cycles a second at full walking speed, which
is brisk and visible — with a per-frame ceiling on top so a burst of speed
cannot alias it either. The feet slide slightly for it. Sliding feet beat
invisible ones.

`SWING`, `LIFT` and `FOLD` were also raised: fifteen degrees of thigh is under a
pixel of foot travel in chase view, which is the only place anybody sees this.

**`FLY_STEP=1`** now reports what the gait is being fed — stance, centimetres a
frame, phase, and whether it thinks it is stepping. Added because the difference
between "the signal is zero" and "the movement is too fast to see" is not
something either of us could tell by looking, and I guessed wrong twice.

## Wings in the air

A held sweep was a stuck decal. There is a buzz on top of it now: twenty-seven
beats a second, small amplitude, applied *after* the smoothing — a shiver eased
at `POSE_RATE` is no shiver at all — with the two wings running a half beat
apart, because a pair in perfect step reads as one object. Aliasing a small
amplitude is what a blur looks like; aliasing a large one is a slow flap, which
is what twelve beats a second looked like.

## Three segments, and why the top one never moved

Brett: "you do know that the fly leg has three segments right?" Femur, tibia,
tarsus. Two bones was under-building it, and the missing third is also why the
top of the leg looked dead.

Two separate faults, both in the weighting, both now impossible to miss because
**the tool prints where the weight landed and complains when a bone is starved**:

**The hip feather was an absolute distance.** 0.085 units, against legs 0.17 to
0.23 long — so it covered the whole top segment and handed it to the body. The
femur ended up with seven per cent of its own leg. It is a fraction of the leg's
length now.

**The leg was parameterised by projection onto a straight chord.** A fly's leg is
bent, so more than half the vertices projected past the last joint: femur 7–20%,
tarsus 53–72%. Measuring from the root instead followed the bend and got it to
12–25% / 50–66% — better and still wrong, because the mesh is denser at the foot
than along the femur. The *distance* was fair and the *vertex count* was not.

Ranking the vertices by distance and banding by rank fixes it: every leg now
reads femur 30%, tibia 36%, tarsus 26%, body 8%. Equal shares of the geometry is
what decides whether a bone can be *seen* to move, which is the only thing that
matters here.

The joints are placed on the centroid of the vertices at that depth, so the chain
follows the leg round its bend instead of cutting the corner.

Driving them: the femur swings and lifts, the tibia folds through the return, and
the tarsus gives some of the fold back so the foot arrives flat rather than
tucked under the leg it hangs from.

**Process note.** Three string-replace patches in a row silently missed because
the file's text differed from what I assumed by a couple of words — "does not
tear" against "does not tear open" — and one of them left the build broken in a
way that looked like a logic error. Anchor on a unique fragment and scan for the
block's end; do not hard-code line numbers or trust remembered prose.

## The femur was turning twenty-five degrees and could not move

Reported four times as a top half that does not animate. Three of my four
explanations were wrong, and the fourth only arrived after logging the one
number that mattered:

    femur driven  25.2 deg sweep

The bone was turning the whole time. **A hinge cannot move the geometry sitting
on its own axis.** The femur bone was planted exactly at the leg root, so the top
of the leg was pinned at the pivot by construction and only the far end swung —
which is exactly what "the bottom animates and the top doesn't" describes, and
no amount of weight or amplitude fixes it.

Real insects have a coxa for this: the pivot is *inboard*, in the body, and the
whole visible leg swings from it. The tool plants the femur's pivot a third of a
leg-length toward the middle of the body now, and the leg swings as one.

### On method

Every wrong guess here was a guess about the *cause* made from a picture. The
weight shares were a real bug and fixing them was right, but they were never the
reported bug — and I only knew that once the drive angle was printed. Three
diagnostics now exist because of this one problem:

- the tool prints where the weight landed and complains about a starved bone
- `FLY_STEP=1` prints stance, travel, phase and the femur's driven angle
- `FLY_GAIT=<phase>` poses the cycle for a capture

The general shape: **when a report and a capture disagree, the thing to measure
is the signal in between, not the two ends.** Amplitude, weight and rate were all
downstream of "is the bone turning, and can turning it move anything".

One more thing the log showed: under a capture `travelled` is 0.0000 cm a frame,
because nothing presses a key. `stepping` only reaches one because `FLY_GAIT`
forces it. Captures can prove the pose is wired; only play proves the gait runs.

## The axis was wrong, and a picture with circles on it said so

Brett circled the parts that were not moving: on each leg, a short stub at the
body and the long femur beyond it. Everything distal to that moved.

**Which way a limb has to turn depends on which way it points.** A fly's legs
splay *outward*, not down. Sweeping them about the body's lateral axis — which is
right for an arm hanging beside a torso — lifts a laterally-reaching leg instead
of swinging it, and lifting is nearly invisible from above. The tibia and tarsus
*do* point downward, so that same rotation swung them properly, which is exactly
why they alone appeared animated. A leg that reaches sideways protracts about the
**vertical**.

Folding likewise happens in the leg's own plane, which for a leg reaching out and
down is about the fore-and-aft axis, not the lateral one.

The short stub at the body was the hip feather: every vertex inside it is one the
body owns and the leg cannot move. Sixteen per cent of each leg was pinned there;
it is nine now.

### What this problem cost, and what it bought

Five rounds, four wrong explanations, and every wrong one was a guess about the
cause made from looking at a render. What actually resolved it, in order:

1. printing where the skin weight landed → caught two real weighting bugs
2. printing the femur's driven angle → proved the bone was turning all along
3. Brett's annotated screenshot → localised it to *which segments*, which is
   what finally identified the axis

The general rule earned here: **when a report and a capture disagree, measure the
signal between them.** And when a diagnostic viewpoint cannot show the thing being
judged, that is a fault in the tooling, not a reason to squint. `FLY_INSPECT` now
takes an elevation — `FLY_INSPECT=35:42` — because a fore-and-aft swing is
invisible from the side of the swing, exactly as a twenty-degree stoop was
invisible head-on.

## Landing by letting go, legs in the air, and faster wings

Three of Brett's asks, and one bug found on the way.

**Letting go is landing.** A fly that stops flying does not hover — it sinks, and
a sinking fly is looking for somewhere to put its feet. So `Intent::land` is now
true whenever no thrust is asked for, and touching any key flies again. `F` and
the right button still ask outright, which is what you want when aiming at a
particular windowsill at speed. Holding *descend* counts as flying: driving
yourself down is not drifting down.

**Legs are carried, not frozen.** A fly holds its forelegs folded up and forward
under the head, its middle pair tucked back along the body, and its hind pair
trailing out behind — and brings all six forward on approach, which is the
landing gear anybody has watched go down on a windowsill. That second half came
free: reaching for a surface is already what `Intent::land` means, so the legs
untuck exactly when the fly starts looking for somewhere to land. `FLY_TUCK=1`
forces it, because a capture is always perched.

**A mirroring bug, found while adding the rows.** The sweep is a turn about the
body's vertical, and one rotation about a shared axis sends the two sides
opposite ways: the same angle that swings a right leg forward swings a left leg
back. Two legs in the same tripod were protracting in opposite directions. The
sweep is mirrored by side now.

**Wings.** A housefly beats at 180–220 Hz. At sixty frames that is 0.3 samples a
beat, so there is no honest way to show it — and raising the rate past thirty
makes it look *slower*, because it aliases down: forty-five reads as fifteen.
Speed therefore comes from less visible travel and more blur, which is also why a
real fly's wings look like nothing but a haze. `FLY_BUZZ=<hz>` and
`FLY_SHIVER=<radians>` are tunable without a rebuild, because past the physics
this is taste and taste needs trying.

### The smear axis, and four captures wasted on not measuring

The blur scales the wing across its chord, and I did not know which local axis
that was. I tried X, then X-or-Z by side, then the reverse, reading each capture
as "one wing broad, one a spike" and inventing a mirrored-bone theory to explain
it. Then I weighed each wing's own vertices into its bone's frame: extents
x=0.550, y=0.421, z=0.264 on one and x=0.493, y=0.425, z=0.296 on the other.
Span is X, chord is Y, Z is the thickness of a membrane — and **both wings agree**,
so they were never mirrored at all.

The "spike" was a wing seen edge-on. Three of the four captures I was reasoning
from showed nothing of the kind, and I had never once rendered the wings with the
smear switched off to see what unscaled looked like. Establish the baseline
before reading a difference.

### And then the smear had to go entirely

Brett: "the wings look like they are double their normal length while flying".
Right, and the measurement above says why, if it is read properly rather than
skimmed for a maximum. A fly's wing is nearer three to one than four to three —
0.55 by 0.42 is not a wing shape at all. **The membrane lies diagonally in bone
space**, so none of the three axes is its span and scaling any of them stretches
it lengthwise.

Doing it properly needs a scale along an arbitrary axis, and a `Transform` cannot
express one: its scale is axis-aligned and applied *before* its rotation, so
`R·S` can never be `R·S·R⁻¹`. It would take a second bone or a shader.

So there is no blur. The wings stay their true size, sweep out with effort, and
shiver at twenty-nine a second; `FLY_SMEAR=<n>` exists mainly to demonstrate the
above. Two lessons, and the second is the one I keep relearning: a measurement
tells you what it measured, not what you hoped — reading "largest extent" as
"the span" was the same shortcut as reading "the bone is turning" as "the leg is
moving".

## The coffee table and two end tables

Brett's models, straight out of 3daistudio rather than through Opificium — which
turns out to be the whole difference. An Opificium export carries an
`opificium-fit` node that centres the mesh in a unit box and then lifts and
scales it to real size; the couch and armchair arrive that way and just work. A
raw export does not: it is centred on its own origin in a unit box, so it sits
half underground and is one metre along its longest side whatever it is a model
of. The coffee table came in eighteen centimetres into the floorboards and the
end tables forty-four, all three the size of a wardrobe.

`use_raw_model` takes the one measurement the file cannot supply — the real
length of the longest side — and works out the rest. Scaling is arithmetic; the
lift is *measured*, by standing the model on the floor once its own collision
exists. Writing the lift down here would be a number taken off one afternoon's
file and silently wrong after the next export, which is the same mistake as the
sofa coordinates and the anatomically honest stride.

Results: coffee table 112 × 62 × 40 with its top at 40, end tables 54 × 44 × 47
with tops at 47. The clutter that stands on them was authored against the
generated tables at 42 and 54 and has been moved to the models' real tops — two
centimetres of daylight under a stack of books reads as wrong without anybody
being able to say why.

`Solid::settle` is the general form: any model can ask to be stood on the floor.

## A daughter, and one set of clips for the family

Brett added `characters/daughter/daughter-walk.glb`. It is **the same rig as the
father** — forty-one bones, identical names, identical nesting — and it contains
**no animation at all**: no `animations` key, seven accessors against his hundred
and thirty-three, despite having been animated in 3daistudio. Something between
the animating and the export dropped it.

That turned out not to matter, and the reason is worth keeping. A clip addresses
a bone by a hash of the *name path* from the animation root down to it. Same
names in the same nesting hash to the same ids, so **his clips are hers**. One
set of animations can move the whole household.

What was missing was the wiring, not the clip. Bevy builds an `AnimationPlayer`
and the per-bone `AnimationTargetId`s only when a glTF actually contains
animations, so there was nothing on her for a borrowed clip to drive and she
stood still while he walked about. `wire_up_borrowed_bones` builds them: forty-
four bones, a player of her own, and she plays his idle.

`folk.rs` no longer knows which resident is the father. `HOUSEHOLD` is a table —
model, clip folders, height, room, spot, facing — and everything else is derived.
It knew, for as long as there was one person, and every hard-coded thing about
him was a thing to untangle the moment a second body arrived.

She stands 138 cm to his 178, and the stoop and forearm-roll corrections apply to
her for free, because they are keyed on bone names and she has his bones.

### Still to do

- She wanders her bedroom and he wanders the great room. Neither can leave the
  room they are in, because a straight line is the only path either can test.
- The turntable picks whichever person it finds first, which is now arbitrary.

## The fly's head, and looking where you are going

Brett asked whether the head was rigged. Measured: not usefully. Seventy-eight
per cent of the head's geometry belongs to `bone_7`, which is the body, and the
`tripo::Head_0` the rig does supply owns eight per cent of it — seventy-five
vertices in a patch whose z range stops dead at the centreline. Turning that
turns a lopsided scrap.

So the tool grew a head, and is now `tools/rig-the-fly.py` rather than
`rig-the-legs.py`. **Where the neck is was measured, not guessed**: sliced across
the body, the vertex count collapses to 71 at x≈0.17 and climbs back to 547 at
the eyes. That trough is the neck; everything forward of it is head, and the
pivot goes *at* the neck rather than in the middle of the head, or turning it
swings the head sideways instead of rotating it.

It leads rather than follows, which is the fly-like part: through a turn the head
arrives at the new heading before the body has finished swinging to it, and in a
drift it points along the *travel* rather than along the body. Fifty-five per cent
of the way, capped at twenty-six degrees — a head that swivels to look down its
own flank is a bird.

### Two bugs, and the same lesson twice

**The head swallowed both front legs — a hundred per cent of them.** The exclusion
test asked each vertex for its dominant bone, and read the weights *as they were
before the leg pass rewrote them*, so no leg vertex looked like a leg vertex. The
leg pass now records what it claimed in a set, which is the only version that
cannot go stale.

**`FLY_LOOK` was never in the build.** A string replacement missed — the file said
`let look` where I had written `let mut look` — so the switch silently did
nothing, and the first capture pair was identical for a completely different
reason than the one I was investigating. The log said `0 deg` and that is what
told me; without it I would have gone looking at weights again.

That is the fourth time this session an edit has silently missed because I
matched on remembered prose. Anchor on something unique and short, or edit by
line, and *check the change landed* before reasoning about its effect.

## Decision: Blender for models, Rust for architecture

Brett: "I am okay using blender instead of Opificium", and "Opificium is in alpha
too. Maybe its just not ready for a project like this".

`CLAUDE.md` is updated, deliberately rather than by drift, because the old law
named Opificium specifically and said the house was generated in Rust. The line
that replaces it splits on a boundary that had been emerging on its own anyway:

**Architecture stays generated.** Two load-bearing reasons. A wall *is* a box, so
its oriented-box collision is exact rather than an approximation — the couch needs
10,695 triangles to describe a shape a box cannot, and modelling the house would
take it from about four thousand exact boxes to several hundred thousand collision
triangles, landing that cost directly on the pixel-perfect-walking problem. And
`house::audit` can only refuse to build what it can measure: room minimums,
ceiling heights, the envelope, the traversal flood-fill, a picture with no wall
behind it. Those laws caught the shelf standing a metre outside the east wall.
A hand-modelled house has no such check.

**Contents become models.** Boxes are wrong for anything without flat faces, and
that is exactly furniture, upholstery, people, plants and clutter. Straight lines
and right angles are *correct* for a building and wrong for a sofa.

Blender is live and confirmed working: 5.2.0 LTS, code execution, PolyHaven
enabled (strongest on textures and HDRIs rather than models).
