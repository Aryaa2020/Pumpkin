# Vanilla Java 26.1 Entity-Parity Tracker

This document tracks progress toward feature parity with vanilla Minecraft Java 26.1
(protocol 775, "Tiny Takeover") for everything entity-related: spawning, pathfinding,
AI/goals, combat, per-mob behavior, conversions, trading, and raids.

Each item is sized to be a single reviewable PR. References cite Mojang class names
(via Mojmap / Yarn) where helpful. Items are ordered by dependency: foundation pieces
first so dependent behavior can rely on them.

Legend: `[ ]` = not started, `[~]` = in progress (PR open), `[x]` = verified by repo owner.

---

## Phase 0 — Audit-fallout bug fixes

These are concrete bugs surfaced by reading the current code; each is small and self-contained.

### Blaze (`entity/ai/goal/blaze_attack.rs`)
- [ ] **0.3** Read `Attributes::FOLLOW_RANGE` instead of hardcoded `48.0` (vanilla `Mob#getAttributeValue(FOLLOW_RANGE)`).
- [ ] **0.4** `is_alive` check on target in `can_start` and `should_continue` (vanilla `Goal#canUse` requires `LivingEntity#isAlive`).
- [ ] **0.5** Line-of-sight raycast before fireball (vanilla `LivingEntity#hasLineOfSight`).
- [ ] **0.6** Apply melee damage in close-range branch (vanilla `Mob#doHurtTarget`).

### Melee attack (`entity/ai/goal/melee_attack.rs`)
- [ ] **0.1** `can_start` must verify the navigator can compute a path to the target (vanilla `MeleeAttackGoal#canUse` → `canReach`).
- [ ] **0.2** Add line-of-sight raycast before swinging (vanilla `LivingEntity#hasLineOfSight`).

### Armor stand (`entity/decoration/armor_stand.rs`)
- [ ] **0.7** Drop equipped items on break (vanilla `ArmorStand#dropAllDeathLoot`).
- [ ] **0.8** Honor `IGNITES_ARMOR_STANDS` / `BURNS_ARMOR_STANDS` damage-type flags.
- [ ] **0.9** Emit `GameEvent::ENTITY_DIE` on break.
- [ ] **0.10** Use oak-plank break particles (vanilla uses block-state particle data).

### Misc
- [ ] **0.11** `LivingEntity::pickup` distance gating (vanilla restricts to ~1.0 blocks).
- [ ] **0.12** Replace hardcoded `NIGHT_START` with environment-attribute `MONSTERS_BURN` lookup; ensure undead burning is dimension- and light-correct.
- [ ] **0.13** Creeper-ignite goal: line-of-sight check (currently TODO at `creeper_ignite.rs:78`).
- [ ] **0.14** `SmallFireballEntity::on_hit` (Block branch) must honor `mobGriefing` and only place fire on **air** blocks. Currently overwrites stone/dirt/etc. with fire (vanilla `AbstractHurtingProjectile#onHitBlock`).
- [ ] **0.15** Blaze melee ignites target for 4 seconds (vanilla `Blaze#doHurtTarget`).
- [ ] **0.16** `ActiveTargetGoal::find_closest_target` filters out creative + spectator players (vanilla `EntitySelector.NO_CREATIVE_OR_SPECTATOR`).
- [ ] **0.17** `MobEntity::try_attack` returns `bool` indicating whether the hit landed, so post-hit hooks (ignite, knockback enchant, etc.) can be conditional.

---

## Phase 1 — Core infrastructure

These are reusable building blocks that subsequent phases consume.

- [ ] **1.1** `Mob::can_reach(target)` helper using the existing `Navigator` and `Path::can_reach`.
- [ ] **1.2** `LivingEntity::has_line_of_sight(target)` raycast helper (eye-pos to eye-pos, stops on `is_solid`).
- [ ] **1.3** Attribute audit: every goal that references range/speed/damage reads from `Attributes::*`, not constants.
- [ ] **1.4** `NeutralMob` trait: shared anger system (`anger_time`, `anger_target`, `persistent_anger_target`, `stop_anger_on_target_death`). Vanilla: `net.minecraft.world.entity.NeutralMob`. Consumers: zombified piglin, wolf, enderman, bee, llama, polar bear, panda.
- [ ] **1.5** `ConvertibleMob` trait: shared conversion timer + entity-type swap with NBT carryover. Vanilla: `net.minecraft.world.entity.monster.ZombieVillager#startConverting`, `Zombie#convertVillagerToZombieVillager`. Consumers: drowned, zombie villager, hoglin→zoglin, pig→zombified piglin, mooshroom shear, frog→variant.
- [ ] **1.6** Minimal `Brain` / `MemoryModule` subset (vanilla `net.minecraft.world.entity.ai.Brain`). Required for villager behavior packages, piglin admiration, frog jump, warden anger management. Initial scope: memory storage + activity switching; full schedules are later phases.

---

## Phase 2 — Pathfinding completeness

Walk pathfinding exists. Other movement modes are missing.

- [ ] **2.1** `SwimNodeEvaluator` (vanilla `SwimNodeEvaluator`) for fish, squid, guardian.
- [ ] **2.2** `AmphibiousNodeEvaluator` for turtle, frog, axolotl, drowned.
- [ ] **2.3** `FlyNodeEvaluator` (3D) for parrot, bee, allay, vex, ghast, phantom.
- [ ] **2.4** `ClimbNodeEvaluator` extension for spider, cave spider (vertical-on-walls cost).
- [ ] **2.5** Wire each mob to its correct `PathNavigation` subclass.

---

## Phase 3 — Combat completeness

- [ ] **3.1** `RangedAttackGoal` (vanilla `RangedAttackGoal`): generic ranged-attack scaffold.
- [ ] **3.2** `BowAttackGoal` with charge-up (vanilla `RangedBowAttackGoal` for skeletons).
- [ ] **3.3** Damage calc audit: armor reduction with `armor_toughness`, enchantment-protection caps, magic-damage path bypassing armor, `DamageType` flags.
- [ ] **3.4** Knockback formula (vanilla `LivingEntity#knockback` resists/adds based on `KNOCKBACK_RESISTANCE`).
- [ ] **3.5** Invulnerability ticks: `hurt_time` / `invulnerable_time` 10-tick window per source (vanilla `LivingEntity#hurtTime` and last-damage-source bypass).
- [ ] **3.6** Shield blocking: full block, axe-disable, arrow deflect.
- [ ] **3.7** Critical hits (mid-fall, not-sprinting, not-blind, +50%).
- [ ] **3.8** Sweeping edge.

---

## Phase 4 — Hostile-mob behavior

- [ ] **4.1a** Zombie → Drowned (water immersion timer 300–600 ticks, equipment swap, trident pickup).
- [ ] **4.1b** Zombie kills Villager → Zombie Villager (Hard always; Normal 50%; profession stash).
- [ ] **4.1c** Zombie Villager → Villager (golden apple + weakness, 100–200 second timer with particles, profession restore).
- [ ] **4.2** Zombified piglin: NeutralMob with collective anger.
- [ ] **4.3** Pig → Zombified Piglin on lightning strike.
- [ ] **4.4** Hoglin → Zoglin in overworld (300-tick timer).
- [ ] **4.5** Skeleton: bow attack goal, arrow trajectory, Stray + Wither variants.
- [ ] **4.6** Stray: slowness arrows.
- [ ] **4.7** Wither skeleton: wither-effect on hit, 2-block-tall path navigation.
- [ ] **4.8** Spider: climb-walls flag + skeleton-rider jockey.
- [ ] **4.9** Cave spider: poison on hit (difficulty-scaled).
- [ ] **4.10** Creeper: charged-by-lightning, swell timer + cancel, ocelot/cat avoid goal.
- [ ] **4.11** Enderman: teleport on damage, water damage, stare-trigger anger, block carry/place, eye-of-ender immunity.
- [ ] **4.12** Endermite: 2-min despawn, summon by enderman teleport / thrown ender-pearl rare chance.
- [ ] **4.13** Witch: potion-throw goals (offensive splash + defensive heal/regen/water-breathing/fire-resist/swiftness).
- [ ] **4.14** Evoker: vex summon, fang-line attack, wololo sheep recolor.
- [ ] **4.15** Vindicator: johnny variant, axe attack.
- [ ] **4.16** Pillager: crossbow shoot, captain banner + Bad Omen on kill, raid wave member.
- [ ] **4.17** Ravager: roar + stun, mounted-rider seat for pillager/illager, charge attack.
- [ ] **4.18** Vex: phase-through-walls, charge attack, despawn after summoner dies.
- [ ] **4.19** Illusioner: clones, blindness arrows.
- [ ] **4.20** Phantom: dive-attack on insomniac players, flap, daylight burning.
- [ ] **4.21** Ghast: fireball goal, ambient drift movement.
- [ ] **4.22** Magma cube / Slime: split on death, jump movement controller.
- [ ] **4.23** Blaze full vanilla parity:
  - [ ] **4.23a** `set_charged` sends `FLAGS_ID` metadata (visible spinning rods during charge-up volley).
  - [ ] **4.23b** Heals 1 HP every 30 ticks while `fire_ticks > 0` or in lava (vanilla `Blaze#aiStep`).
  - [ ] **4.23c** Takes 1 damage per tick in water (vanilla `Blaze#aiStep`).
  - [ ] **4.23d** Ambient smoke particle trail.
  - [ ] **4.23e** Confirm fire-immunity flag is set (vanilla `Blaze#fireImmune = true`).
  - [ ] **4.23f** Blaze rod loot drop (vanilla `entities/blaze.json` loot table).
- [ ] **4.24** Hoglin: melee toss, repel from warped fungus and overworld portals.
- [ ] **4.25** Piglin: hostile-to-non-gold-armor, gold-ingot bartering, hostile to wither_skeleton & zoglin, overworld zombification (15s timer).
- [ ] **4.26** Piglin Brute: bastion guard, no zombification fear.
- [ ] **4.27** Warden: anger system, sniff goal, sonic-boom ranged, dig despawn (60s no anger).
- [ ] **4.28** Breeze: wind-charge projectile, jump pattern.
- [ ] **4.29** Creaking: pale-garden link to creaking heart, freeze-while-watched, immortal until heart broken.
- [ ] **4.30** Guardian / Elder Guardian: laser beam, dolphin's-grace immunity, mining-fatigue III aura (elder).
- [ ] **4.31** Shulker: teleport, homing-bullet projectile, levitation effect, color variants.
- [ ] **4.32** Silverfish: stone block infest, swarm-on-hit.
- [ ] **4.33** Bat: ceiling hang, flee on damage.
- [ ] **4.34** Drowned (post-conversion): trident throw, nautilus shell pickup, baby variant, target villagers + iron golems.

---

## Phase 5 — Passive-mob behavior

- [ ] **5.1** Cow / Mooshroom: milk bucket, mushroom shear (red↔brown lightning), bowl→stew interact, suspicious stew flowers.
- [ ] **5.2** Sheep: shear, dye-on-interact, baby color genetics, regrow wool on eat-grass.
- [ ] **5.3** Pig: saddle, carrot-on-stick steering (lightning conversion in 4.3).
- [ ] **5.4** Chicken: egg laying (6000-tick timer), chicken-jockey detection, baby.
- [ ] **5.5** Wolf: tame (bone), sit, anger inheritance, pack hunt, healing with meat, all 9 vanilla 26.1 color variants, howl.
- [ ] **5.6** Cat: tame (raw fish), sit, gift on owner wakeup, sleep on bed near owner, hiss at phantoms.
- [ ] **5.7** Ocelot: tempt with fish, trust state.
- [ ] **5.8** Fox: day-sleep, snow-fox biome variant, item-in-mouth, vex-chickens, jump-over-fences flag.
- [ ] **5.9** Rabbit: killer-rabbit variant, eat carrots from farms, evade.
- [ ] **5.10** Bee: pollinate, return to hive, sting (lose stinger), pollinated state, honey-block harvest mood.
- [ ] **5.11** Frog: tongue-eat (slime / magma cube → variant slimeball / froglight), jump.
- [ ] **5.12** Tadpole: grow into frog after 24000 ticks, biome decides variant.
- [ ] **5.13** Axolotl: play-dead on damage, attack drowned/squid/fish, regen aura, blue rare-breed.
- [ ] **5.14** Allay: pickup item, follow noteblock, duplicate on amethyst, sing.
- [ ] **5.15** Goat: ram, screaming variant, horn drop on ram.
- [ ] **5.16** Camel: dash, sit, double-rider.
- [ ] **5.17** Sniffer: dig (torchflower / pitcher seed), egg, baby.
- [ ] **5.18** Panda: gene system (lazy/aggressive/playful/worried/brown/weak), bamboo eat, sneeze.
- [ ] **5.19** Polar bear: protect cub, hostile if cub nearby.
- [ ] **5.20** Iron golem: village-defense, poppy gift, anger, attack monsters.
- [ ] **5.21** Snow golem: snow trail, pumpkin shear, water/desert damage, snowball ranged (no damage).
- [ ] **5.22** Squid: ink cloud on damage, water-only swim.
- [ ] **5.23** Glow squid: glow-ink, dim on damage.
- [ ] **5.24** Cod / Salmon / Tropical fish / Pufferfish: schooling, bucket, pufferfish puff state + poison aura.
- [ ] **5.25** Dolphin: dolphin's-grace effect aura, lead-to-treasure, surface for breath.
- [ ] **5.26** Turtle: home-beach memory, lay egg, scute drop on baby growth, slow on land.
- [ ] **5.27** Strider: lava swim, saddle, warped-fungus-on-stick, shaking when not in lava.
- [ ] **5.28** Horse / Donkey / Mule / Skeleton horse / Zombie horse: tame, breed, color/markings, jump strength, leashing, chest on donkey/mule.
- [ ] **5.29** Llama / Trader llama: caravan, spit, carpet decoration, inventory, strength.
- [ ] **5.30** Parrot: dance to jukebox, mimic mob sounds, sit on shoulder, cookie kills.
- [ ] **5.31** Wandering trader: spawn timer, drink invisibility/milk, lead trader llamas, despawn after 40 minutes.
- [ ] **5.32** Villager: profession from workstation, gossip propagation, restock 2x/day, level-up XP, golem summon, mate, panic, hide-indoors-at-night, raid response.
- [ ] **5.33** Zombie villager: profession persistence in NBT (covered by 4.1c).

---

## Phase 6 — Spawning

- [ ] **6.1** Audit per-biome mob lists against `worldgen/biome/*.json` (every vanilla biome).
- [ ] **6.2** Light-level + sky-light gating per vanilla 26.1 hostile rules.
- [ ] **6.3** Mob-cap-per-category verification (monster / creature / ambient / water / underground / axolotl).
- [ ] **6.4** Structure spawns: nether fortress, bastion remnant, ocean monument, woodland mansion, pillager outpost, igloo basement, stronghold, ancient city, trial chamber.
- [ ] **6.5** Mob spawner: verify `spawn_potentials`, `spawn_count`, `max_nearby_entities`, `spawn_range`, `delay` range.
- [ ] **6.6** Trial spawner / vault spawner.
- [ ] **6.7** Spawn egg item: spawn entity at click position with NBT.
- [ ] **6.8** Pillager patrol spawning every 5500–6500 ticks.
- [ ] **6.9** Wandering trader spawn every 24000 ticks (1/10 chance).
- [ ] **6.10** Phantom spawn over insomniac players (>72000 ticks since sleep).

---

## Phase 7 — Raids

- [ ] **7.1** Bad Omen → Raid Omen on entering village center.
- [ ] **7.2** `Raid` manager per village.
- [ ] **7.3** Raid wave composition (5 on Easy/Normal, 7 on Hard) with bonus waves.
- [ ] **7.4** Captain banner mechanics + bad-omen-on-kill.
- [ ] **7.5** Raid victory → Hero of the Village effect.

---

## Phase 8 — Boss & finishing pass

- [ ] **8.1** Ender Dragon AI parity audit + fixes.
- [ ] **8.2** Wither AI parity audit + fixes.
- [ ] **8.3** Final sweep: every vanilla mob spawn-tested in its expected biome / structure.

---

## How items are shipped

1. Each item is a single PR against `master`.
2. PR title format: `parity(<phase>.<n>): <short description>`.
3. PR description includes: vanilla reference, what changed, how to test in-game.
4. After PR merges, this file is updated to tick the box and the next PR begins.

## How to test a shipped item

Each PR description contains a "How to test" section. General steps to run Pumpkin locally:

```bash
git clone https://github.com/Aryaa2020/Pumpkin
cd Pumpkin
git checkout <branch-name>          # the branch the PR is on
cargo run --release                  # binds to 0.0.0.0:25565 by default
```

Then connect a vanilla 1.21.x – 26.1 Java client to `localhost`. The handshake accepts
clients from `1.20.5` through `26.1` per `pumpkin-data/src/generated/packet.rs`.
