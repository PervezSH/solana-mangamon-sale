use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_mangamon_sale {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub authorized_sale_account: Account<'info, AuthorizedSaleAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AuthorizedSaleAccount {
    // Pubkey of Admin
    pub admin: Pubkey,
    // Price of idoToken in payToken value based on ratio
    pub ido_token_price_ratio: u64,
    pub ido_token_price_multiplier: u64,
    // Initial Allocation on Ido Tokens, if 0, then there is no initial allocation
    pub initial_percentage_allocation_ido_tokens: u64,

    // Total amount of tokens to be sold, is Set at the funding
    pub tokens_for_sale: u128,

    // Start and End date for the eligible addresses to buy their tokens/funds
    pub start_date_funding: i64,
    pub end_date_funding: i64,
    // End date until when token can be claimed
    pub start_date_of_claiming_tokens: i64,
    pub end_date_of_claiming_tokens: i64,

    // If the IDO token has been funded to the contract
    pub is_ido_token_funded_to_contract: bool,
    // To keep track of the ido Cancellation
    pub is_funding_canceled: bool,
    // Enables the payment only to be in one transaction
    pub in_one_transaction: bool,
    // Enable claiming
    pub is_claiming_open: bool,
}
