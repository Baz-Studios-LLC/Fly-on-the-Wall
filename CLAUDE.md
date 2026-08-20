# Fly on the Wall - Claude Instructions

Read `README.md` before changing the project. Treat the repository's existing
behavior and documented decisions as the source of truth.

## Project Laws

- This is a Rust 2024 and Bevy 0.19 project. Verify APIs against that version.
- One world unit is one centimetre.
- The house is built to `assets/FloorPlan.jpg`, at the real dimensions printed
  on it. That plan replaced a 15-by-15-foot minimum and a uniform 9-foot
  ceiling, both of which it breaks everywhere: only its great room would have
  passed the minimum, and its ceilings are deliberately 10 feet through the
  living areas and 9 in the secondary bedrooms and baths.
- **A law was not deleted here, it was made stricter.** "At least fifteen feet"
  only ever caught a room that was small. "Matches the plan" catches a room that
  is the wrong size in either direction, in the wrong place, or missing — so
  `house::audit` checks every room against the plan's own figures, and a
  mismatch refuses to build exactly as the old minimum did.
- Ceiling heights come from the plan, per room, and are not uniform. The
  variation is worth having: a fly crossing from a 9-foot bedroom into a 10-foot
  great room should feel the volume change.
- The current flight model is approved. Preserve its feel and controls unless a
  house change exposes a concrete collision or traversal bug.
- Rendering geometry and collision geometry must agree. A visible solid should
  behave as solid, and deliberate fly-sized openings must remain traversable.
- The **architecture** is generated in this game with Rust, Bevy primitives,
  meshes, and mathematics: walls, floors, ceilings, doors, windows, trim, stairs
  and roof. Two reasons, and both are load-bearing. A wall *is* a box, so its
  oriented-box collision is exact rather than an approximation — a modelled wall
  would need a triangle hull, and the house would go from about four thousand
  exact boxes to several hundred thousand collision triangles. And `house::audit`
  can only refuse to build what it can measure: room minimums, ceiling heights,
  the envelope, the traversal flood-fill, a picture with no wall behind it. A
  hand-modelled house has no such check, and every mistake in it becomes
  something a person has to happen to notice.
- The **contents** may be made models: furniture, upholstery, people, plants,
  clutter. Boxes are wrong for anything without flat faces, which is exactly
  what these are. **Blender is the tool for making them** — Opificium is in
  alpha and not ready for this project. Anything from 3daistudio arrives centred
  in a unit box and wants `use_raw_model` with its real length; anything
  normalised at source wants plain `use_model`.
- Build reusable procedural constructors for repeated forms, but do not hide
  useful artistic control behind a prematurely generic generation framework.
- Preserve existing diagnostic controls such as `FLY_PLAN`, `FLY_CAPTURE`, and
  `FLY_INSPECT`; extend them when stronger visual validation needs it.
- Do not add a physics engine or major dependency without a demonstrated need.

## Autonomous Development

When working autonomously on a goal, act as a persistent senior game developer,
technical artist, and level designer. Do not interpret one completed prop, room,
or code task as completion of a broader visual goal.

Use this continuous loop:

1. Read the active goal, relevant code, and `.claude/AUTONOMOUS_LOG.md`.
2. Run the game or inspect fresh captures to establish the current visual state.
3. Identify the highest-value unresolved problem within the goal. Prefer the
   weakest player-visible area, unless correctness is blocking it.
4. Decide whether the issue belongs in procedural architecture, prop generation,
   materials, lighting, collision, camera/capture tooling, or performance.
5. Research established solutions when the problem is difficult, unfamiliar,
   poorly documented, or has resisted two substantive approaches.
6. Implement one coherent improvement that is large enough to matter visually.
7. Format, compile, test, run, and capture the result as appropriate.
8. Inspect the rendered evidence. Check composition, scale, construction,
   materials, lighting, intersections, floating objects, and fly traversal.
9. Fix problems introduced or revealed by the change.
10. Record significant work and validation in the autonomous log.
11. Select the next highest-value improvement and continue.

For an explicitly continuous goal, keep cycling until the user interrupts the
session or a genuinely consequential decision cannot be inferred. Never create
busywork merely to remain active: each cycle must produce or validate a
meaningful improvement.

## House Quality Standard

Judge the house as a complete, coherent, lived-in environment, not a prop demo.
Continue improving it room by room and then revisit earlier rooms in the context
of the whole house.

Evaluate all of the following:

- Complete architecture: floors, walls, ceilings, roof where visible, doors,
  windows, frames, trim, stairs, thresholds, and believable room connections.
- No occupiable room smaller than 15 feet by 15 feet of clear interior floor
  space; verify generated room bounds rather than relying on nominal dimensions.
- Consistent 9-foot (`274.32 cm`) finished floor-to-ceiling room height.
- Room identity and function: every room should read clearly from its contents
  and layout without labels.
- Essential furnishings and fixtures appropriate to each room.
- Secondary objects and restrained clutter that make the home inhabited.
- Coherent dimensions, materials, palette, construction language, and period.
- Believable daylight, practical light sources, shadowing, and exposure.
- Human-scale usability and fly-scale routes, gaps, landing surfaces, sheltered
  spaces, and interesting sightlines.
- No accidental overlaps, blocked doors, unsupported objects, paper-thin forms,
  obvious missing surfaces, or repetitive placeholder arrangements.
- Reasonable draw cost and entity/light counts, measured before optimization.

Use hierarchical procedural construction. Prefer small reusable helpers for
recognizable forms such as boards, frames, legs, cushions, shelves, handles, and
fixtures; compose those into authored room arrangements. Pure random scattering
is not a substitute for design. Any randomness must be seeded and reproducible.

Do not judge visual work from source code alone. Generate fresh screenshots from
useful viewpoints, including a whole-house plan and representative eye-level or
fly-level room views. Add deterministic capture viewpoints when existing tools
cannot show the work clearly. Inspect the images before choosing the next task.

## Development Priorities

Prefer work in this order:

1. Correctness and crashes
2. Missing or broken house structure
3. Player-visible quality and room completeness
4. Believability, traversal, and consistency
5. Lighting and materials
6. Measured performance
7. Maintainability and refactoring

Do not prioritize code cleanliness over visible quality unless the code is
actively preventing further iteration.

## Validation

Use the checks appropriate to each change, including:

- `cargo fmt --check`
- `cargo check`
- relevant `cargo test` targets
- runtime validation
- `FLY_PLAN=1` for whole-house composition
- `FLY_CAPTURE=<path>` with suitable delay and viewpoint for visual evidence
- collision and traversal checks at fly scale
- entity, mesh, material, light, frame-time, or memory measurements when relevant

Compilation is not proof that visual or behavioral work succeeded. Do not claim
a visual improvement without inspecting a fresh capture, or a traversal fix
without exercising the affected route.

## External Research

Research online when a problem is technically difficult, unfamiliar, related to
obscure Bevy behavior, or likely to have established prior art. After two failed
substantive approaches to the same problem, stop guessing and research it.

Prefer official documentation, official repositories and issue trackers,
maintainer discussions, technical papers, and strong engineering references.
For rendering, procedural modeling, lighting, collision, spatial indexing, or
performance, consider useful techniques from other engines and graphics fields,
then translate the underlying idea into idiomatic Rust and Bevy 0.19. Record
research-derived decisions in the autonomous log.

## Guardrails

Do not:

- change the approved flight feel as part of an aesthetic pass
- invent unrelated game mechanics, characters, objectives, HUD, or progression
- model the architecture by hand, for the two reasons under Project Laws
- use Opificium: it is in alpha and not ready for this project
- replace working systems merely because another architecture looks cleaner
- make large rewrites when a focused extension will support continued work
- remove working features without strong justification
- optimize without measurements
- introduce non-deterministic generation that makes regressions hard to compare
- silently accept a worse image because tests pass
- stop after producing one impressive object while adjacent rooms remain empty

Routine visual and implementation choices do not require approval. Infer a
coherent domestic style from the existing game and continue. Ask only when a
choice would substantially alter the game's identity, require replacing a major
established system, or cannot reasonably be inferred from available evidence.

## Autonomous Log

Maintain `.claude/AUTONOMOUS_LOG.md` during autonomous work. Keep it concise and
record significant discoveries, changes, visual captures, tests, measurements,
research decisions, deferrals, and genuine user decisions. Consult it before
starting another improvement so completed work is not repeatedly rediscovered.

Permanent rules in this file define how to work. The current prompt or `/goal`
defines what to work on.

## Git Checkpoints

After a coherent improvement has passed its relevant checks and visual review,
make a focused local commit. Do not bundle unrelated experiments into it, revert
user work, rewrite published history, or push unless the user explicitly asks.
