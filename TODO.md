# TODO — Hatchspire

This backlog is limited to implementation, test, harness, and documentation work that
can be completed by an AI coding agent. Subjective playtest tuning, visual approval,
and open-ended external toolchain decisions are intentionally excluded.

## Polish

- Add a reusable tooltip primitive that supports pointer hover and a visible touch path.
- Add contextual tooltips to town map, building upgrade, facility entry, and menu controls.
- Add actionable validation messages for town building, shop trade, and NPC greeting failures.
- Add actionable validation messages for hatchery, stable, breeding, and workshop failures.
- Add tower and combat failure/recovery messages with a visible touch action for each.

## Balance tooling

- Define a serializable combat replay format containing the RNG seed, roster, encounter,
  commands, and expected outcome.
- Record player commands and the RNG seed while a combat encounter is running.
- Implement a replay runner that reconstructs an encounter and reports the first mismatch.
- Add deterministic replay tests for victory, defeat, fleeing, and item use.

## Verification captures

- Extend the capture harness with seeded stable, breeding grove, workshop, shop, and
  combat scenes.
- Capture town and hatchery verification screens in `docs/verification/`.
- Capture stable, breeding grove, and workshop verification screens in
  `docs/verification/`.
- Capture shop, tower, and combat verification screens in `docs/verification/`.

## Engineering — integration coverage

- Add an integration test for the hatchery care and hatching flow.
- Add an integration test for stable roster and recovery actions.
- Add an integration test for shop trades and purchase validation.
- Add an integration test for tower preparation, movement, return, and rewards.
- Add an integration test for combat commands, resolution, and recovery.
- Add an integration test for town purchases and building progression.

## Engineering — shared configuration

- Move monster stat curves into shared typed game data and add integrity checks.
- Move combat cooldown definitions into shared typed game data and add integrity checks.
- Move shop inventory and trade costs into shared typed game data and add integrity checks.
- Move tower rewards into shared typed game data and add integrity checks.

## Engineering — command boundaries

- Standardise explicit town commands for building, trade, greeting, save, and navigation.
- Move remaining town progression mutations behind the town command/reducer boundary.
- Standardise explicit combat commands for attacks, skills, items, defence, and fleeing.
- Move remaining combat progression mutations behind the combat command/reducer boundary.
- Add regression tests proving rendering does not mutate town or combat progression.
