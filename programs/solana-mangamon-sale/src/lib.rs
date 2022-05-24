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

    // Setters
    /// Change the initial percentage of token allocation to be claimed
    pub fn set_initial_percentage_allocation_ido_tokens(
        ctx: Context<UpdateAuhorizedSaleAccount>,
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
        emit!(ChangedInitialPercentageAllocationIdoTokens {
            admin: *ctx.accounts.admin.key,
            old_initial_percentage_allocation_ido_tokens:
                _old_initial_percentage_allocation_ido_tokens,
            initial_percentage_allocation_ido_tokens: authorized_sale_account
                .initial_percentage_allocation_ido_tokens
        });
        Ok(())
    }
    /// Set if the claiming is enabled or not
    pub fn enable_claiming(
        ctx: Context<UpdateAuhorizedSaleAccount>,
        _is_claiming_open: bool,
        _start_date_of_claiming_tokens: i64,
    ) -> Result<()> {
        let authorized_sale_account = &mut ctx.accounts.authorized_sale_account;
        let _old_is_claiming_open = authorized_sale_account.is_claiming_open;

        authorized_sale_account.is_claiming_open = _is_claiming_open;
        authorized_sale_account.start_date_of_claiming_tokens = _start_date_of_claiming_tokens;

        emit!(ChangedIsClaimingOpen {
            admin: *ctx.accounts.admin.key,
            old_is_claiming_open: _old_is_claiming_open,
            is_claiming_open: authorized_sale_account.is_claiming_open
        });
        Ok(())
    }
    /// Set the end date for tokens to be claimed by all buyers
    pub fn set_end_date_of_claiming_tokens(
        ctx: Context<UpdateAuhorizedSaleAccount>,
        _end_date_of_claiming_tokens: i64,
    ) -> Result<()> {
        let authorized_sale_account = &mut ctx.accounts.authorized_sale_account;
        assert_eq!(
            authorized_sale_account.is_claiming_open, false,
            "Claiming is already enabled"
        );
        let _old_end_date_of_claiming_tokens = authorized_sale_account.end_date_of_claiming_tokens;
        authorized_sale_account.end_date_of_claiming_tokens = _end_date_of_claiming_tokens;
        emit!(ChangedEndDateOfClaimingTokens {
            admin: *ctx.accounts.admin.key,
            old_end_date_of_claiming_tokens: _old_end_date_of_claiming_tokens,
            end_date_of_claiming_tokens: authorized_sale_account.end_date_of_claiming_tokens
        });
        Ok(())
    }

    // BusinessLogic
    /// Give the programAddress the ido tokens to be sold
    pub fn fund_to_contract(
        ctx: Context<UpdateBothSaleAccount>,
        _amount_in_ido_tokens: u128,
    ) -> Result<()> {
        assert!(ctx.accounts.is_funding_closed());
        assert!(ctx.accounts.is_funding_not_canceled_by_admin());
        let authorized_sale_account = &mut ctx.accounts.authorized_sale_account;
        let sale_account = &mut ctx.accounts.sale_account;
        assert_eq!(
            authorized_sale_account.is_ido_token_funded_to_contract, false,
            "Already funded tokens"
        );
        assert!(
            sale_account.total_allocated_ido_tokens <= _amount_in_ido_tokens,
            "You should at least match the totalAllocatedIdoTokens"
        );
        // check if admin have enough tokens to fund to the contract
        // transfer funds
        authorized_sale_account.tokens_for_sale = _amount_in_ido_tokens;
        authorized_sale_account.is_ido_token_funded_to_contract = true;
        Ok(())
    }
    /// Withdraw Pay Tokens from contract Only withdraw Pay tokens after the funding has ended
    pub fn withdraw_pay_tokens(
        ctx: Context<UpdateBothSaleAccount>,
        _pay_tokens_to_withdraw: u128,
    ) -> Result<()> {
        assert!(ctx.accounts.is_funding_closed());
        // withdraw pay tokens
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

/// Validation struct for updating fields of AuthorizedSaleAccount
#[derive(Accounts)]
pub struct UpdateAuhorizedSaleAccount<'info> {
    #[account(mut, has_one = admin)]
    pub authorized_sale_account: Account<'info, AuthorizedSaleAccount>,
    pub admin: Signer<'info>,
}

/// Validation struct for updating fields of both AuthorizedSaleAccount and SaleAccount
#[derive(Accounts)]
pub struct UpdateBothSaleAccount<'info> {
    #[account(mut, has_one = admin)]
    pub authorized_sale_account: Account<'info, AuthorizedSaleAccount>,
    #[account(mut)]
    pub sale_account: Account<'info, SaleAccount>,
    pub admin: Signer<'info>,
}
impl<'info> UpdateBothSaleAccount<'info> {
    /// Check if the contract has been funded enough sale tokens
    pub fn is_ido_token_funded(&self) -> bool {
        assert!(
            self.authorized_sale_account.is_ido_token_funded_to_contract,
            "The contract did not receive the IDO tokens"
        );
        true
    }
    /// Check if the Funding period is open
    pub fn is_funding_open_and_running(&self) -> bool {
        let now_ts = Clock::get().unwrap().unix_timestamp;
        assert!(
            now_ts >= self.authorized_sale_account.start_date_funding
                && now_ts <= self.authorized_sale_account.end_date_funding,
            "The Funding Period is not Open"
        );
        true
    }
    /// Check if the Funding has ended
    pub fn is_funding_closed(&self) -> bool {
        let now_ts = Clock::get().unwrap().unix_timestamp;
        assert!(
            now_ts > self.authorized_sale_account.end_date_funding,
            "The Funding Period has not ended"
        );
        true
    }
    /// Check if the Funding has been canceled
    pub fn is_funding_canceled_by_admin(&self) -> bool {
        assert_eq!(
            self.authorized_sale_account.is_funding_canceled, true,
            "Funding has not been canceled"
        );
        true
    }
    /// Check if the Funding has not been canceled
    pub fn is_funding_not_canceled_by_admin(&self) -> bool {
        assert_eq!(
            self.authorized_sale_account.is_funding_canceled, false,
            "Funding has been canceled"
        );
        true
    }
}

// Accouunts
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

// Events
/// Event that will get emitted on buying IDO tokens
#[event]
pub struct BoughtIDOTokens {
    pub buyer: Pubkey,
    pub spend_pay_tokens: u128,
    pub ido_tokens_to_get: u128,
    pub timestamp: i64,
}
/// Event that will get emitted on claiming IDO tokens
#[event]
pub struct ClaimedIDOTokens {
    pub buyer: Pubkey,
    pub ido_tokens_to_get: u128,
}

// Logging
/// Event that will get emmited on changing end date, untill token can be claimed
#[event]
pub struct ChangedEndDateOfClaimingTokens {
    pub admin: Pubkey,
    pub old_end_date_of_claiming_tokens: i64,
    pub end_date_of_claiming_tokens: i64,
}
/// Event that will get emmited on changing end date, untill token can be claimed
#[event]
pub struct ChangedIsClaimingOpen {
    pub admin: Pubkey,
    pub old_is_claiming_open: bool,
    pub is_claiming_open: bool,
}
/// Event that will get emmited on changing initial percentage allocation of IDO tokens
#[event]
pub struct ChangedInitialPercentageAllocationIdoTokens {
    pub admin: Pubkey,
    pub old_initial_percentage_allocation_ido_tokens: u8,
    pub initial_percentage_allocation_ido_tokens: u8,
}
// Event that will get emmited on changing IDO token address
// pub struct ChangedIdoTokenAddress {
//     pub admin: Pubkey,
//     pub old_ito_token: bool,
//     pub new_ido_token: bool,
// }
