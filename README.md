# P Senshi Program

## Overview

A Solana on-chain program for a fantasy validator league built with [Pinocchio](https://github.com/anza-xyz/pinocchio). Players stake JitoSOL on individual validators across epoch-based seasons and compete for a share of the prize pool (including accrued yield).

Each season is scoped to a **single validator** (vote account) and an epoch range. Players enter by paying an entry fee, and rewards are distributed after the season is scored and settled.

## Architecture

### Program ID

```
SenPmWgTAKKhCxCAtKJLkV5yz7YW8VKQgUpTE5rEFYb
```

### Accounts

#### Config

Global configuration account.

| Field     | Type      | Size |
|-----------|-----------|------|
| authority | Address   | 32   |

- **PDA**: `["config"]`
- **Discriminator**: `[1, 0, 0, 0, 0, 0, 0, 0]`

#### Season

Per-validator, per-epoch season account.

| Field         | Type      | Size |
|---------------|-----------|------|
| authority     | Address   | 32   |
| vault         | Address   | 32   |
| epoch_start   | u64       | 8    |
| epoch_end     | u64       | 8    |
| prize_pool    | u64       | 8    |
| entry_fee     | u64       | 8    |
| total_entries | u32       | 4    |
| status        | u8        | 1    |
| bump          | u8        | 1    |
| reserved      | [u8; 128] | 128  |

- **PDA**: `["season", vote_account, epoch_start.to_be_bytes()]`
- **Discriminator**: `[2, 0, 0, 0, 0, 0, 0, 0]`
- **Total size**: 230 bytes (+ 8 byte discriminator)

#### Entry

Per-player entry within a season.

| Field      | Type     | Size |
|------------|----------|------|
| player     | Address  | 32   |
| has_score  | u8       | 1    |
| score      | u64      | 8    |
| has_reward | u8       | 1    |
| reward     | u64      | 8    |
| claimed    | u8       | 1    |
| bump       | u8       | 1    |
| reserved   | [u8; 64] | 64   |

- **PDA**: `["entry", season_pda, player]`
- **Discriminator**: `[3, 0, 0, 0, 0, 0, 0, 0]`
- **Total size**: 116 bytes (+ 8 byte discriminator)

### Season Lifecycle

```
Open -> Locked -> Scoring -> Settled
```

| Status  | Description                                    |
|---------|------------------------------------------------|
| Open    | Accepting new entries                           |
| Locked  | Entries closed, epoch window has started         |
| Scoring | Scores are being submitted by the authority     |
| Settled | Rewards distributed, players can claim          |

### Instructions

| Tag | Instruction        | Data                                     |
|-----|--------------------|------------------------------------------|
| 0   | InitializeConfig   | (none)                                   |
| 1   | InitializeSeason   | entry_fee (8) + epoch_start (8) + epoch_end (8) |
| 2   | EnterSeason        | epoch_start (8)                          |
| 3   | LockSeason         | epoch_start (8)                          |
| 4   | SubmitScores       | epoch_start (8) + score (8)              |
| 5   | SettleSeason       | epoch_start (8)                          |
| 6   | ClaimReward        | epoch_start (8)                          |

#### 0 - InitializeConfig

Creates the global Config PDA. The payer becomes the authority.

**Accounts**: `[config (w), payer (s/w), system_program]`

#### 1 - InitializeSeason

Creates a Season PDA for a specific vote account and epoch range.

**Accounts**: `[season (w), payer (s/w), vote_account, vault, system_program]`

#### 2 - EnterSeason

Player enters an open season. Transfers entry fee to the vault and creates an Entry PDA.

**Accounts**: `[season (w), entry (w), player (s/w), vote_account, player_token (w), vault (w), token_program, system_program]`

#### 3 - LockSeason

Authority locks the season once the target epoch has begun. Requires `clock.epoch >= epoch_start`.

**Accounts**: `[season (w), authority (s), vote_account]`

#### 4 - SubmitScores

Authority submits a score for a single entry. Transitions season to Scoring on first call.

**Accounts**: `[season (w), authority (s), vote_account, entry (w)]`

#### 5 - SettleSeason

Authority settles the season after scoring. Updates prize pool from vault balance and transitions to Settled. Requires `clock.epoch > epoch_end`.

**Accounts**: `[season (w), authority (s), vote_account, vault]`

#### 6 - ClaimReward

Player claims their reward after settlement. Transfers reward from vault to player's token account.

**Accounts**: `[season, entry (w), player (s), vote_account, vault (w), vault_authority, player_token (w), token_program]`

### Error Codes

| Code | Name              | Description                              |
|------|-------------------|------------------------------------------|
| 0    | ArithmeticError   | Arithmetic under/overflow                |
| 1    | SeasonNotOpen     | Season is not open for entries           |
| 2    | InvalidTransition | Invalid season status transition         |
| 3    | EpochNotReached   | Target epoch has not been reached yet    |
| 4    | EpochNotEnded     | Season epoch has not ended yet           |
| 5    | NotSettled        | Season is not settled                    |
| 6    | AlreadyClaimed    | Reward has already been claimed          |
| 7    | NoReward          | No reward assigned to this entry         |
