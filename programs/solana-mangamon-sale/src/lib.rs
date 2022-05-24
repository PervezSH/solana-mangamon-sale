use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_mangamon_sale {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        _ido_token_price_ratio: u64,
        _start_date_funding: i64,
        _end_date_funding: i64,
        _end_date_of_claiming_tokens: i64,
        _claiming_initial_percentage: u8,
        _in_one_transaction: bool,
    ) -> Result<()> {
        // Get a mutable reference to the accounts
        let authorized_sale_account = &mut ctx.accounts.authorized_sale_account;
        let sale_account = &mut ctx.accounts.sale_account;
        // Get a reference to the user
        let user = &ctx.accounts.user;

        // Initialize fields of authorized_sale_account
        // Set admin
        authorized_sale_account.admin = *user.key;

        // Set initial sale token price
        authorized_sale_account.ido_token_price_ratio = _ido_token_price_ratio;
        authorized_sale_account.ido_token_price_multiplier = 10000;

        // Set amount of tokens to be sold
        authorized_sale_account.tokens_for_sale = 0;

        // Set starting and ending dates of funding
        authorized_sale_account.start_date_funding = _start_date_funding;
        authorized_sale_account.end_date_funding = _end_date_funding;

        // Tokens to be claimed until
        authorized_sale_account.end_date_of_claiming_tokens = _end_date_of_claiming_tokens;

        // Default value, until funds have been made
        authorized_sale_account.is_ido_token_funded_to_contract = false;
        // Set Ido Cancellation
        authorized_sale_account.is_funding_canceled = false;
        // Set IDO tokens need to be payed in one transaction
        authorized_sale_account.in_one_transaction = _in_one_transaction;
        // Set claiming
        authorized_sale_account.is_claiming_open = false;

        // Initialize fields of sale_account
        // Set count of total pay tokens spend
        sale_account.total_spend_pay_tokens = 0;
        // Set count of total IDO tokens sold
        sale_account.total_allocated_ido_tokens = 0;
        // Set investor count
        sale_account.investor_count = 0;
        Ok(())
    }

    /// Change the initial percentage of token allocation to be claimed
    pub fn set_initial_percentage_allocation_ido_tokens(
        ctx: Context<SetInitialPercentageAllocationIdoTokens>,
        _percentage: u8,
    ) -> Result<()> {
        let authorized_sale_account = &mut ctx.accounts.authorized_sale_account;
        assert_eq!(
            authorized_sale_account.is_claiming_open, false,
            "Claiming is already enabled"
        );
        assert!(
            _percentage <= 100,
            "You cannot give more than 100 percent of the token allocation"
        );
        let _old_initial_percentage_allocation_ido_tokens =
            authorized_sale_account.initial_percentage_allocation_ido_tokens;
        authorized_sale_account.initial_percentage_allocation_ido_tokens = _percentage;
        // emit event
        Ok(())
    }

    /// Builds a PDA hashmap, and initializes its fields
    pub fn creat_buyer_info(
        ctx: Context<CreatBuyerInfo>,
        _spend_pay_tokens: u128,
        _ido_tokens_to_get: u128,
    ) -> Result<()> {
        println!("I'm in");
        let buyer_info = &mut ctx.accounts.buyer_info;
        buyer_info.spend_pay_tokens = _spend_pay_tokens;
        buyer_info.ido_tokens_to_get = _ido_tokens_to_get;
        buyer_info.ido_tokens_claimed = 0;
        buyer_info.has_claimed_pay_tokens = false;
        buyer_info.bump = *ctx.bumps.get("buyer_info").unwrap();
        Ok(())
    }
}

/// Validation struct for initialize
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 101)]
    pub authorized_sale_account: Account<'info, AuthorizedSaleAccount>,
    #[account(init, payer = user, space = 8 + 3244)]
    pub sale_account: Account<'info, SaleAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Validation struct for set_initial_percentage_allocation_ido_tokens
#[derive(Accounts)]
pub struct SetInitialPercentageAllocationIdoTokens<'info> {
    #[account(mut, has_one = admin)]
    pub authorized_sale_account: Account<'info, AuthorizedSaleAccount>,
    pub admin: Signer<'info>,
}

/// Validation struct for creat_buyer_info
#[derive(Accounts)]
pub struct CreatBuyerInfo<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + 50,
        seeds = [b"buyer-info", user.key().as_ref()],
        bump
    )]
    pub buyer_info: Account<'info, BuyerInfo>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AuthorizedSaleAccount {
    // Pubkey of Admin
    pub admin: Pubkey, // 32 bytes

    // Price of idoToken in payToken value based on ratio
    pub ido_token_price_ratio: u64,      // 8 bytes
    pub ido_token_price_multiplier: u64, // 8 bytes
    // Initial Allocation on Ido Tokens, if 0, then there is no initial allocation
    pub initial_percentage_allocation_ido_tokens: u8, // 1 bytes

    // Total amount of tokens to be sold, is Set at the funding
    pub tokens_for_sale: u128, // 16 bytes

    // Start and End date for the eligible addresses to buy their tokens/funds
    pub start_date_funding: i64, // 8 bytes
    pub end_date_funding: i64,   // 8 bytes
    // End date until when token can be claimed
    pub start_date_of_claiming_tokens: i64, // 8 bytes
    pub end_date_of_claiming_tokens: i64,   // 8 bytes

    // If the IDO token has been funded to the contract
    pub is_ido_token_funded_to_contract: bool, // 1 bytes
    // To keep track of the ido Cancellation
    pub is_funding_canceled: bool, // 1 bytes
    // Enables the payment only to be in one transaction
    pub in_one_transaction: bool, // 1 bytes
    // Enable claiming
    pub is_claiming_open: bool, // 1 bytes
} // 108 bytes

#[account]
pub struct SaleAccount {
    // Spend Pay Count
    pub total_spend_pay_tokens: u128, // 16 bytes
    // Sold tokens count
    pub total_allocated_ido_tokens: u128, // 16 bytes
    // Investors count
    pub investor_count: u64, // 8 bytes
    // Array to keep track of all the buyers
    pub buyers_list: Vec<Pubkey>, // upto 100 buyers, (4 + 100 * 32) bytes = 3204 bytes
} // 3244 bytes

#[account]
pub struct BuyerInfo {
    // keep track of how many pay tokens have been spend by the buyer
    spend_pay_tokens: u128, // 16 bytes
    // keep tack of how many IDO tokens the buyer should get
    ido_tokens_to_get: u128, // 16 bytes
    // keep tack of how many IDO tokens the buyer has claimed
    ido_tokens_claimed: u128, // 16 bytes
    // keep track if the buyer has claimed the pay tokens spend on IDO cancel
    has_claimed_pay_tokens: bool, // 1 bytes
    bump: u8,                     // 1 bytes
} // 50 bytes
