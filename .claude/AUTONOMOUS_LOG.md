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

## Deferred

- Family simulation, needs, danger, death, objectives, progression, and HUD are
  outside the active house-and-lighting goal.
- Flight and camera redesign are deferred unless house validation reveals a
  specific regression that blocks traversal or inspection.
- Hinged door leaves. Openings are honest holes for now; a swinging door is its
  own piece of work and the greybox still has the only one.

## Next

- Then: fly-scale routes, undersides and landing surfaces; restrained clutter.
