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
- A fresh procedural-house visual baseline still needs to be captured and judged.

## Continuous Backlog

- Establish a deterministic procedural-house entry point and visual baseline.
- Complete the architectural shell and make every room reachable and legible.
- Keep every occupiable room at or above 15 feet by 15 feet (`457.2 cm x 457.2
  cm`) measured between finished interior surfaces.
- Use 9-foot (`274.32 cm`) ceilings measured from finished floor to finished
  ceiling throughout occupiable rooms.
- Give every room a clear domestic function and coherent circulation.
- Build reusable procedural furniture and fixture constructors from mathematical
  forms, then author complete room arrangements with them.
- Add secondary props and restrained lived-in clutter after primary layouts work.
- Develop coherent materials, color variation, windows, practical fixtures,
  daylight, shadows, and exposure.
- Inspect fly-level routes, landing surfaces, hiding places, gaps, and sightlines.
- Revisit the weakest-looking room or object after every validated pass.
- Measure and control entity, mesh, material, and shadow-light cost as detail grows.

This backlog is cyclical, not a completion checklist. Once every area has a first
pass, compare the whole house and begin a stronger refinement pass.

## Completed

- Repository architecture and current validation tools were surveyed for the
  autonomous workflow.
- The goal was explicitly constrained to in-game mathematical generation rather
  than Opificium authoring.

## Validation

- None yet for procedural house output; establish fresh plan and room captures
  before recording the first implementation pass.

## Research

- None yet.

## Deferred

- Family simulation, needs, danger, death, objectives, progression, and HUD are
  outside the active house-and-lighting goal.
- Flight and camera redesign are deferred unless house validation reveals a
  specific regression that blocks traversal or inspection.

## Needs User Decision

- None. Make conservative, coherent visual decisions and continue working.
