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

## Deferred

- Family simulation, needs, danger, death, objectives, progression, and HUD are
  outside the active house-and-lighting goal.
- Flight and camera redesign are deferred unless house validation reveals a
  specific regression that blocks traversal or inspection.
- Hinged door leaves. Openings are honest holes for now; a swinging door is its
  own piece of work and the greybox still has the only one.

## Next

- Then: fly-scale routes, undersides and landing surfaces; restrained clutter.
