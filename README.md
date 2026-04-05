# P Senshi Program

## Overview


```rs
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("VLEAGUExxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

#[program]
pub mod validator_league {
    use super::*;

    /// Player claims their reward after settlement
    pub fn claim_reward(ctx: Context<ClaimReward>, season_id: u64) -> Result<()> {
        let entry = &mut ctx.accounts.entry;
        let season = &ctx.accounts.season;

        require!(season.status == SeasonStatus::Settled, VLError::NotSettled);
        require!(!entry.claimed, VLError::AlreadyClaimed);
        require!(entry.reward.is_some(), VLError::NoReward);

        let reward = entry.reward.unwrap();
        entry.claimed = true;

        // Transfer from vault PDA to player
        let seeds = &[
            b"vault",
            season_id.to_le_bytes().as_ref(),
            &[ctx.accounts.season.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.player_jitosol.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                signer,
            ),
            reward,
        )?;

        emit!(RewardClaimed {
            season_id,
            player: ctx.accounts.player.key(),
            reward,
        });

        Ok(())
    }
}



// ---------------------------------------------------------------------------
// Contexts
// ---------------------------------------------------------------------------



#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct KeeperAction<'info> {
    #[account(
        mut,
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump = season.bump,
        has_one = authority,
    )]
    pub season: Account<'info, Season>,

    #[account(
        mut,
        constraint = vault.key() == season.vault,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub authority: Signer<'info>,
}


#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct ClaimReward<'info> {
    #[account(
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump = season.bump,
    )]
    pub season: Account<'info, Season>,

    #[account(
        mut,
        seeds = [b"entry", season_id.to_le_bytes().as_ref(), player.key().as_ref()],
        bump = entry.bump,
        has_one = player,
    )]
    pub entry: Account<'info, Entry>,

    #[account(
        mut,
        constraint = vault.key() == season.vault,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for vault
    #[account(
        seeds = [b"vault", season_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = player_jitosol.owner == player.key(),
    )]
    pub player_jitosol: Account<'info, TokenAccount>,

    pub player: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct RewardClaimed {
    pub season_id: u64,
    pub player: Pubkey,
    pub reward: u64,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[error_code]
pub enum VLError {
    #[msg("Season is not open for entries")]
    SeasonNotOpen,
    #[msg("Invalid roster size")]
    InvalidRosterSize,
    #[msg("Invalid entry fee")]
    InvalidEntryFee,
    #[msg("Duplicate validator in roster")]
    DuplicateValidator,
    #[msg("Invalid status transition")]
    InvalidTransition,
    #[msg("Target epoch not yet reached")]
    EpochNotReached,
    #[msg("Competition epoch has not ended")]
    EpochNotEnded,
    #[msg("Season not yet settled")]
    NotSettled,
    #[msg("Reward already claimed")]
    AlreadyClaimed,
    #[msg("No reward to claim")]
    NoReward,
    #[msg("Array length mismatch")]
    LengthMismatch,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pad_validators(validators: Vec<Pubkey>, roster_size: u8) -> [Pubkey; 10] {
    let mut padded = [Pubkey::default(); 10];
    for (i, v) in validators.iter().enumerate().take(roster_size as usize) {
        padded[i] = *v;
    }
    padded
}
```
