# Autonomic Platform — Combinatorial Maximalism Feature Spec

## Current State
- **GLOBE screen**: deck.gl 3D planet, drag-rotate, click nodes to inspect, live telemetry, trade arcs
- **ARENA screen**: Three.js Tron battle arena, 5v5 agent swarms, lanes, real-time combat log
- **DECK screen**: TCG-style AutoML models (Mythic/Rare/Uncommon/Common rarity)
- **OPS screen**: NPC operators, quests, self-healing incidents, achievements
- **Gamification HUD**: Level + XP, battle pass progress, streak counter, autonomy combo

---

## 9 High-Impact Expansions (Combinatorial Stacking)

### 1. **Leaderboards** (3 axes)
- **Global ELO**: all players ranked by win rate / autonomy score
- **Per-Agent**: rank each of 5 blue agents independently (vs all players)
- **Per-Model**: rank TCG cards by deployments + win rate
- **UI**: Leaderboard screen (new nav tab) with seasonal resets, cosmetic badges for top 100

### 2. **Battle Pass Seasons**
- 100-tier track (weekly + daily challenges)
- Free path + premium path (cosmetic cosmetics unlock, not power)
- Seasonal reset every 4 weeks; carry-over cosmetics + badges
- **Feeds into**: prestige system (can ascend for 2x XP multiplier)

### 3. **Prestige / Ascension**
- At level 100, player can "prestige" → resets to level 1 with 2x XP multiplier
- Each prestige tier unlocks cosmetic prestige badge + higher ELO "prestige rank"
- Prestige level is visible on player card + leaderboard
- **Feeds into**: cosmetics shop (prestige-exclusive skins)

### 4. **Card Synergies + Combo Trees**
- Every card has 1–3 "synergy keywords" (e.g. "neural-net", "swarm-logic", "cascade")
- Deploy 3+ cards with matching keywords → +15% ATK/DEF for that squad
- **Synergy tree UI**: show active combos in deck view, highlight in battle
- Unlock new synergies by deploying 50+ times together
- **Feeds into**: leaderboard for "most synergies activated"

### 5. **Agent Talent Trees**
- Each of 5 agents has a 3-tier unlock tree (9 talents total)
- Unlock by deploying that agent 100/500/1000 times
- Talents: +10% ATK, +8% DEF, reflect 15% damage, heal 5 HP/turn, +20% crit, etc.
- **UI**: talent grid in OPS screen; drag to slot into agent's loadout (3 active at a time)
- **Feeds into**: guild system (share optimized loadouts)

### 6. **Threat Waveforms + Raid Bosses**
- **Waveforms**: animated threat curve on COMMAND globe; shows incoming disruption magnitude/ETA
- **Raid bosses**: quarterly "world threats" (e.g. "Dark Network Intrusion", "Supply Chain Collapse")
- Raid requires 5+ guild members co-op; 3-phase fight scaling 3000 HP
- Defeat = global loot (cosmetics, XP multiplier, card shards)
- **UI**: raid progress bar + co-op roster in OPS

### 7. **Cosmetics Shop**
- Seasonal skins for agents (neon, retro, holographic themes)
- Card backs for TCG (animated, particle effects)
- Globe themes (dark matter, plasma, aurora)
- Currency: earned via battle pass / prestige
- **UI**: shop screen (new nav tab); preview before buy; show equipped cosmetics on cards/agents

### 8. **Guild / Clan System**
- Create/join guilds (max 20 members)
- Guild vault (shared quest rewards, cosmetics, prestige badges)
- Guild wars (asynchronous: attack/defend leaderboard ranking)
- Guild quests (co-op objectives for XP multiplier)
- **UI**: guild card in nav; roster in OPS; guild chat log (simple message feed)

### 9. **Procedural Roguelike Campaigns**
- "Endless Ops" mode: procedurally-generated quest chains (20–50 escalating fights)
- Each fight: random agent pool + modifiers (2x threat, 1.5x XP, invincible first turn, etc.)
- Rewards: cumulative loot per run; prestige-scaling (10x multiplier on 5th prestige)
- **UI**: campaign map (branching tree of nodes); run stats (longest streak, most XP earned)

---

## Integration Points

### Simulation Loop
- Every 300ms tick:
  1. **Threat wave**: advance incoming raid/world-event HPs; trigger waveforms
  2. **Agent actions**: resolve talents + synergies; apply cosmetic skins
  3. **Quest generation**: procedural campaign nodes based on prestige level
  4. **Leaderboard**: update global ELO from live wins
  5. **Battle Pass**: progress weekly challenges (auto-detect milestone for XP boosts)

### Render Loop
- **COMMAND**: overlay waveform curve + raid progress on globe
- **ARENA**: render agent talent icons + synergy glow + cosmetic skins on units
- **DECK**: show card synergy keywords + combo highlight; cosmetic card-backs
- **OPS**: talent grid, guild roster, raid co-op phase, campaign map
- **Leaderboard**: (new) rankings + prestige badges + seasonal reset timer

### Data Model Extensions
```js
// Player
{ level, prestige, eloRating, seasonBattlePassTier, guildId, cosmetics: {}, talents: {} }

// Agent
{ talents: [Talent, Talent, Talent], skinCosmetic, synergies: [keyword1, keyword2] }

// Card (TCG model)
{ synergies: [keyword], cosmetic: cardBackId, deployCount }

// Guild
{ name, members: [playerId], vault: {}, warRanking, questLog }

// Raid Boss
{ name, hp, maxHp, phaseNum, cosplayers: [playerId] }

// Roguelike Campaign
{ runId, prestige, nodes: [{modifiers, threat}], loot: [] }
```

---

## Delivery Path

1. **Leaderboards** (2hrs): add `eloRating` to player state; compute from arena wins; add leaderboard screen with 3 tabs
2. **Battle Pass** (2hrs): add tier system to HUD; challenge tracking; reward unlock animation
3. **Prestige** (1hr): add reset button; 2x XP multiplier; cosmetic badge
4. **Synergies** (2hrs): keyword arrays on cards; live combo detection; highlight glow in arena
5. **Talent Trees** (3hrs): 9-talent unlock grids per agent; loadout UI; stat application
6. **Waveforms** (2hrs): animated threat curve overlay on globe; phase indicator in ops
7. **Cosmetics** (2hrs): shop UI; skin swaps on agents/cards/globe; preview system
8. **Guilds** (3hrs): guild creation, roster, vault, guild wars async ranking, simple chat
9. **Roguelike** (2hrs): procedural node tree generation, modifiers, run leaderboard

**Total**: ~19 hours of implementation. Ready to code.

---

## Design System Hooks

All cosmetics, colors, and glow effects inherit from the existing palette:
- **Rarity colors**: Mythic (violet), Rare (cyan), Uncommon (emerald), Common (dim)
- **Prestige badges**: gradient overlays + star icons
- **Synergy glow**: additive blending, pulsing at 2Hz
- **Raid phase**: health bar color shifts (green → amber → magenta as HP drains)

No new design tokens needed; everything stacks on the existing neon/glassmorphism aesthetic.
