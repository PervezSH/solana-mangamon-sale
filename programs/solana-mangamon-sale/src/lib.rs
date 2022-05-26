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
        assert!(
            _start_date_funding < _end_date_funding,
            "The starting date of the funding should be before the end date of the funding"
        );
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
        ctx: Context<AdminOnlyUpdate>,
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
        ctx: Context<AdminOnlyUpdate>,
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
        ctx: Context<AdminOnlyUpdate>,
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

    // Getters
    /// Returns a list of all buyers (wallet addresses)
    pub fn get_buyers(ctx: Context<ReadAccounts>) -> Result<Vec<Pubkey>> {
        Ok(ctx.accounts.sale_account.buyers_list.clone())
    }
    /// Returns boolean of the wallet address when he is a buyer or not
    pub fn is_buyer(ctx: Context<ReadAccounts>, _buyer: Pubkey) -> Result<bool> {
        let buyers_list = &ctx.accounts.sale_account.buyers_list;
        for buyer in buyers_list {
            if buyer.to_bytes() == _buyer.to_bytes() {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Get total tokens bought by msg.sender, and total tokens spent
    pub fn get_total_ido_tokens_bought_and_pay_tokens_spend(
        ctx: Context<ReadBuyerInfoAndAccounts>,
        _buyer: Pubkey,
    ) -> Result<Vec<u128>> {
        let buyer_info = &ctx.accounts.buyer_info;
        let _total_bought_ido_tokens = buyer_info.ido_tokens_to_get;
        let _total_spend_pay_tokens = buyer_info.spend_pay_tokens;
        Ok(vec![_total_bought_ido_tokens, _total_spend_pay_tokens])
    }
    /// Get total tokens bought, and total tokens spent
    pub fn total_ido_tokens_bought_and_pay_tokens_spend(
        ctx: Context<ReadAccounts>,
    ) -> Result<Vec<u128>> {
        let sale_account = &ctx.accounts.sale_account;
        Ok(vec![
            sale_account.total_allocated_ido_tokens,
            sale_account.total_spend_pay_tokens,
        ])
    }
    /// Returns the claimable tokens at this point in time
    pub fn get_claimable_tokens(
        ctx: Context<ReadBuyerInfoAndAccounts>,
        _buyer: Pubkey,
    ) -> Result<u128> {
        let buyer_info = &ctx.accounts.buyer_info;
        let authorized_sale_account = &ctx.accounts.authorized_sale_account;
        let initial_tokens_to_get = ctx
            .accounts
            .calculate_ido_tokens_bought(ctx.accounts.buyer_info.spend_pay_tokens);

        let _seconds_in_total_between_start_and_end_date_claiming_tokens = authorized_sale_account
            .end_date_of_claiming_tokens
            .checked_sub(authorized_sale_account.start_date_of_claiming_tokens)
            .unwrap();
        let mut _ido_tokens_per_second = buyer_info
            .ido_tokens_to_get
            .checked_div(_seconds_in_total_between_start_and_end_date_claiming_tokens as u128)
            .unwrap();
        let mut _total_tokens_to_get = 0u128;

        // Allocate all the tokens, if current time already surpassed the endDateOfClaimingTokens
        if Clock::get().unwrap().unix_timestamp
            > authorized_sale_account.end_date_of_claiming_tokens
        {
            _total_tokens_to_get = buyer_info.ido_tokens_to_get;
        } else if authorized_sale_account.initial_percentage_allocation_ido_tokens > 0 {
            // Calculates the _totalTokensToGet with this percentage
            let _initial_tokens_to_get = initial_tokens_to_get
                .checked_div(100)
                .unwrap()
                .checked_mul(
                    authorized_sale_account.initial_percentage_allocation_ido_tokens as u128,
                )
                .unwrap();
            if Clock::get().unwrap().unix_timestamp
                >= authorized_sale_account.start_date_of_claiming_tokens
            {
                // Removes the initial tokes to get from the total supply tokens to get percentage
                _ido_tokens_per_second = buyer_info
                    .ido_tokens_to_get
                    .checked_sub(_initial_tokens_to_get)
                    .unwrap()
                    .checked_div(
                        _seconds_in_total_between_start_and_end_date_claiming_tokens as u128,
                    )
                    .unwrap();
                // Calculate how many tokens to get since startDateOfClaimingTokens
                let _seconds_passed_since_start = Clock::get()
                    .unwrap()
                    .unix_timestamp
                    .checked_sub(authorized_sale_account.start_date_of_claiming_tokens)
                    .unwrap();
                _total_tokens_to_get = _ido_tokens_per_second
                    .checked_mul(_seconds_passed_since_start as u128)
                    .unwrap();
            }
            // Add the initial tokens to get to the tokens that can be claimed
            _total_tokens_to_get = _total_tokens_to_get
                .checked_add(_initial_tokens_to_get)
                .unwrap();
        } else {
            // End date has not yet been reached
            let _seconds_passed_since_start = Clock::get()
                .unwrap()
                .unix_timestamp
                .checked_sub(authorized_sale_account.start_date_of_claiming_tokens)
                .unwrap();
            _total_tokens_to_get = _ido_tokens_per_second
                .checked_mul(_seconds_passed_since_start as u128)
                .unwrap();
        }
        // Subtract previous already claimed tokens
        _total_tokens_to_get = _total_tokens_to_get
            .checked_sub(buyer_info.ido_tokens_claimed)
            .unwrap();
        Ok(_total_tokens_to_get)
    }

    // BusinessLogic
    /// Calculates how much Payment tokens needed to acquire IDO token allocation
    pub fn calculate_max_payment_token(
        ctx: Context<ReadAccounts>,
        _ido_tokens_to_get: u128,
    ) -> Result<u128> {
        let authorized_sale_account = &ctx.accounts.authorized_sale_account;

        let ido_token_decimal: u128 = 10u128.checked_pow(18 - 2).unwrap();
        let pay_token_token_decimal: u128 = 10u128.checked_pow(6 - 2).unwrap();

        let _ido_tokens_to_get: u128 = _ido_tokens_to_get.checked_div(ido_token_decimal).unwrap(); // 10000000000000000 / 10 ^ 16 = 1

        let ido_token_price_ratio = authorized_sale_account.ido_token_price_ratio as u128;
        let _divide_by_ratio = ido_token_price_ratio
            .checked_mul(pay_token_token_decimal)
            .unwrap(); // (4 * 10 ^ 3) * 10 ^ 4 = 4 * 10 ^ 7

        let mut _amount_in_pay_token = (_ido_tokens_to_get).checked_mul(_divide_by_ratio).unwrap(); // 1 * 4 * 10 ^ 7 = 4 * 10 ^ 7
        let ido_token_price_multiplier = authorized_sale_account.ido_token_price_multiplier as u128;
        _amount_in_pay_token = _amount_in_pay_token
            .checked_div(ido_token_price_multiplier)
            .unwrap(); // (4 * 10 ^ 7) / 10 ^ 4 = 4 * 10 ^ 3 USDC tokens
        Ok(_amount_in_pay_token)
    }
    /// Calculate the amount of Ido Tokens bought
    pub fn calculate_ido_tokens_bought(
        ctx: Context<ReadAccounts>,
        _amount_in_pay_token: u128,
    ) -> Result<u128> {
        let authorized_sale_account = &ctx.accounts.authorized_sale_account;

        let ido_token_decimal: u128 = 10u128.checked_pow(18 - 2).unwrap();
        let pay_token_token_decimal: u128 = 10u128.checked_pow(6 - 2).unwrap();

        let _amount_in_pay_token = _amount_in_pay_token
            .checked_mul(authorized_sale_account.ido_token_price_multiplier as u128)
            .unwrap(); // 250_000_000 * 10_000 = 2_500_000_000_000
        let _divide_by_ratio = (authorized_sale_account.ido_token_price_ratio as u128)
            .checked_mul(pay_token_token_decimal)
            .unwrap(); // 4_000 * 10_000 = 40_000_000
        let mut _ido_tokens_to_get = _amount_in_pay_token.checked_div(_divide_by_ratio).unwrap(); // 2_500_000_000_000 / 40_000_000 = 62_500
        _ido_tokens_to_get = _ido_tokens_to_get.checked_mul(ido_token_decimal).unwrap(); // 62_500 * 10_000_000_000_000_000 = 625_000_000_000_000_000_000
        Ok(_ido_tokens_to_get)
    }
    /// Give the programAddress the ido tokens to be sold
    pub fn fund_to_contract(
        ctx: Context<AdminOnlyUpdate>,
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
        // todo: check if admin have enough tokens to fund to the contract
        // todo: transfer funds
        authorized_sale_account.tokens_for_sale = _amount_in_ido_tokens;
        authorized_sale_account.is_ido_token_funded_to_contract = true;
        Ok(())
    }
    /// Buy Tokens, but not really, just transfer the payment tokens to the Contract
    /// and create a receipt that can later be claimed by the buyer
    pub fn buy(ctx: Context<BuyersOnlyUpdate>, _amount_in_pay_token: u128) -> Result<()> {
        ctx.accounts.is_funding_open_and_running();
        ctx.accounts.is_funding_canceled_by_admin();
        // todo: isLotteryPlayedAndAllocationCalculated
        // todo: onlyWinners
        let amount_in_pay_token = ctx.accounts.calculate_max_payment_token(10000000000000000); // todo: data fron lottery contract
        let ido_tokens_to_buy = ctx
            .accounts
            .calculate_ido_tokens_bought(_amount_in_pay_token);
        let is_buyer = ctx.accounts.is_buyer(*ctx.accounts.user.key);

        assert!(_amount_in_pay_token > 0, "Amount has to be positive");
        // todo: check user has enough pay tokens
        assert!(_amount_in_pay_token <= amount_in_pay_token,
            "You cannot buy more tokens than is allowed according to your lottery allocation calculation"
        );
        let buyer_info = &mut ctx.accounts.buyer_info;
        let final_spend_pay_tokens = buyer_info
            .spend_pay_tokens
            .checked_add(_amount_in_pay_token)
            .unwrap();
        assert!(final_spend_pay_tokens <= amount_in_pay_token,
            "You cannot buy more tokens than is allowed according to your lottery allocation calculation"
        );
        let authorized_sale_account = &ctx.accounts.authorized_sale_account;
        if authorized_sale_account.in_one_transaction {
            assert!(
                amount_in_pay_token == _amount_in_pay_token,
                "You need to buy the entire allocation in one transaction"
            );
        }
        let sale_account = &mut ctx.accounts.sale_account;
        if buyer_info.spend_pay_tokens == 0 {
            sale_account.investor_count = sale_account.investor_count.checked_add(1).unwrap();
        }
        // todo: get paid in pay tokens
        sale_account.total_spend_pay_tokens = sale_account
            .total_spend_pay_tokens
            .checked_add(_amount_in_pay_token)
            .unwrap();
        sale_account.total_allocated_ido_tokens = sale_account
            .total_allocated_ido_tokens
            .checked_add(ido_tokens_to_buy)
            .unwrap();

        buyer_info.spend_pay_tokens = buyer_info
            .spend_pay_tokens
            .checked_add(_amount_in_pay_token)
            .unwrap();
        buyer_info.ido_tokens_to_get = buyer_info
            .ido_tokens_to_get
            .checked_add(ido_tokens_to_buy)
            .unwrap();
        if !is_buyer {
            sale_account.buyers_list.push(*ctx.accounts.user.key);
        }
        emit!(BoughtIDOTokens {
            buyer: *ctx.accounts.user.key,
            spend_pay_tokens: _amount_in_pay_token,
            ido_tokens_to_get: ido_tokens_to_buy,
            timestamp: Clock::get().unwrap().unix_timestamp
        });
        Ok(())
    }
    /// After the Funding period, users are allowed to claim their IDO Tokens
    pub fn claim_tokens(ctx: Context<BuyersOnlyUpdate>) -> Result<()> {
        ctx.accounts.is_funding_closed();
        ctx.accounts.is_funding_not_canceled_by_admin();
        // todo: isLotteryPlayedAndAllocationCalculated

        let is_buyer = ctx.accounts.is_buyer(*ctx.accounts.user.key);
        let initial_tokens_to_get = ctx
            .accounts
            .calculate_ido_tokens_bought(ctx.accounts.buyer_info.spend_pay_tokens);

        let authorized_sale_account = &ctx.accounts.authorized_sale_account;
        assert!(
            authorized_sale_account.is_ido_token_funded_to_contract,
            "Tokens have not been added to the contract YET"
        );
        let buyer_info = &mut ctx.accounts.buyer_info;
        assert!(
            buyer_info.ido_tokens_claimed < buyer_info.ido_tokens_to_get,
            "You have already claimed the tokens"
        );
        assert!(
            authorized_sale_account.is_claiming_open,
            "Cannot claim, you need to wait until claiming is enabled"
        );
        assert!(is_buyer, "You are not a buyer");

        let _seconds_in_total_between_start_and_end_date_claiming_tokens = authorized_sale_account
            .end_date_of_claiming_tokens
            .checked_sub(authorized_sale_account.start_date_of_claiming_tokens)
            .unwrap();
        let mut _ido_tokens_per_second = buyer_info
            .ido_tokens_to_get
            .checked_div(_seconds_in_total_between_start_and_end_date_claiming_tokens as u128)
            .unwrap();
        let mut _total_tokens_to_get = 0u128;

        // Allocate all the tokens, if current time already surpassed the endDateOfClaimingTokens
        if Clock::get().unwrap().unix_timestamp
            > authorized_sale_account.end_date_of_claiming_tokens
        {
            _total_tokens_to_get = buyer_info.ido_tokens_to_get;
        } else if authorized_sale_account.initial_percentage_allocation_ido_tokens > 0 {
            // Calculates the _totalTokensToGet with this percentage
            let _initial_tokens_to_get = initial_tokens_to_get
                .checked_div(100)
                .unwrap()
                .checked_mul(
                    authorized_sale_account.initial_percentage_allocation_ido_tokens as u128,
                )
                .unwrap();
            if Clock::get().unwrap().unix_timestamp
                >= authorized_sale_account.start_date_of_claiming_tokens
            {
                // Removes the initial tokes to get from the total supply tokens to get percentage
                _ido_tokens_per_second = buyer_info
                    .ido_tokens_to_get
                    .checked_sub(_initial_tokens_to_get)
                    .unwrap()
                    .checked_div(
                        _seconds_in_total_between_start_and_end_date_claiming_tokens as u128,
                    )
                    .unwrap();
                // Calculate how many tokens to get since startDateOfClaimingTokens
                let _seconds_passed_since_start = Clock::get()
                    .unwrap()
                    .unix_timestamp
                    .checked_sub(authorized_sale_account.start_date_of_claiming_tokens)
                    .unwrap();
                _total_tokens_to_get = _ido_tokens_per_second
                    .checked_mul(_seconds_passed_since_start as u128)
                    .unwrap();
            }
            // Add the initial tokens to get to the tokens that can be claimed
            _total_tokens_to_get = _total_tokens_to_get
                .checked_add(_initial_tokens_to_get)
                .unwrap();
        } else {
            // End date has not yet been reached
            let _seconds_passed_since_start = Clock::get()
                .unwrap()
                .unix_timestamp
                .checked_sub(authorized_sale_account.start_date_of_claiming_tokens)
                .unwrap();
            _total_tokens_to_get = _ido_tokens_per_second
                .checked_mul(_seconds_passed_since_start as u128)
                .unwrap();
        }
        // Subtract previous already claimed tokens
        _total_tokens_to_get = _total_tokens_to_get
            .checked_sub(buyer_info.ido_tokens_claimed)
            .unwrap();
        // todo: transfer the idoTokens to the msg.sender from the contract
        // Update mapping
        buyer_info.ido_tokens_claimed = buyer_info
            .ido_tokens_claimed
            .checked_add(_total_tokens_to_get)
            .unwrap();
        emit!(ClaimedIDOTokens {
            buyer: *ctx.accounts.user.key,
            ido_tokens_to_get: _total_tokens_to_get
        });
        Ok(())
    }
    /// Withdraw Pay Tokens from contract Only withdraw Pay tokens after the funding has ended
    pub fn withdraw_pay_tokens(
        ctx: Context<AdminOnlyUpdate>,
        _pay_tokens_to_withdraw: u128,
    ) -> Result<()> {
        assert!(ctx.accounts.is_funding_closed());
        // todo: withdraw pay tokens
        Ok(())
    }
    /// Withdraw unsold IDO tokens
    pub fn withdraw_unsold_ido_tokens(
        ctx: Context<AdminOnlyUpdate>,
        _ido_tokens_to_withdraw: u128,
    ) -> Result<()> {
        assert!(ctx.accounts.is_funding_closed());
        // todo: if IDO token is available, withdraw ido tokens
        Ok(())
    }
    /// Cancels the entire sale
    pub fn cancel_ido_sale(ctx: Context<AdminOnlyUpdate>) -> Result<()> {
        let authorized_sale_account = &mut ctx.accounts.authorized_sale_account;
        authorized_sale_account.is_funding_canceled = true;
        Ok(())
    }
    /// Let users claim his payed tokens if ido sale is canceled
    pub fn claim_payed_tokens_on_ido_cancel(ctx: Context<BuyersOnlyUpdate>) -> Result<()> {
        ctx.accounts.is_funding_canceled_by_admin();
        let buyer_info = &mut ctx.accounts.buyer_info;
        assert_eq!(
            buyer_info.has_claimed_pay_tokens, false,
            "You have been refunded already"
        );
        let _pay_tokens_to_return = buyer_info.spend_pay_tokens;
        // Update states
        buyer_info.spend_pay_tokens = 0;
        buyer_info.has_claimed_pay_tokens = true;
        // todo: transfer pay tokens
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

/// Validation struct for updating fields by admin only
#[derive(Accounts)]
pub struct AdminOnlyUpdate<'info> {
    #[account(mut, has_one = admin)]
    pub authorized_sale_account: Account<'info, AuthorizedSaleAccount>,
    #[account(mut)]
    pub sale_account: Account<'info, SaleAccount>,
    pub admin: Signer<'info>,
}
impl<'info> AdminOnlyUpdate<'info> {
    /// Check if the Funding has ended
    pub fn is_funding_closed(&self) -> bool {
        let now_ts = Clock::get().unwrap().unix_timestamp;
        assert!(
            now_ts > self.authorized_sale_account.end_date_funding,
            "The Funding Period has not ended"
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

/// Validation struct for updating fields of SaleAccount and Buyer's info with reference to the AuthorizedSaleAccount
#[derive(Accounts)]
pub struct BuyersOnlyUpdate<'info> {
    pub authorized_sale_account: Account<'info, AuthorizedSaleAccount>,
    #[account(mut)]
    pub sale_account: Account<'info, SaleAccount>,
    #[account(mut, seeds = [b"buyer-info", user.key().as_ref()], bump = buyer_info.bump)]
    pub buyer_info: Account<'info, BuyerInfo>,
    pub user: Signer<'info>,
}
impl<'info> BuyersOnlyUpdate<'info> {
    // Checks
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
    /// Check if the Funding has been canceled
    pub fn is_funding_canceled_by_admin(&self) -> bool {
        assert_eq!(
            self.authorized_sale_account.is_funding_canceled, true,
            "Funding has not been canceled"
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
    /// Check if the Funding has not been canceled
    pub fn is_funding_not_canceled_by_admin(&self) -> bool {
        assert_eq!(
            self.authorized_sale_account.is_funding_canceled, false,
            "Funding has been canceled"
        );
        true
    }
    /// Checks if buyer's in buyer list
    pub fn is_buyer(&self, _buyer: Pubkey) -> bool {
        let buyers_list = &self.sale_account.buyers_list;
        for buyer in buyers_list {
            if buyer.to_bytes() == _buyer.to_bytes() {
                return true;
            }
        }
        false
    }
    // Calculations
    /// Calculates how much Payment tokens needed to acquire IDO token allocation
    pub fn calculate_max_payment_token(&self, _ido_tokens_to_get: u128) -> u128 {
        let authorized_sale_account = &self.authorized_sale_account;

        let ido_token_decimal: u128 = 10u128.checked_pow(18 - 2).unwrap();
        let pay_token_token_decimal: u128 = 10u128.checked_pow(6 - 2).unwrap();

        let _ido_tokens_to_get: u128 = _ido_tokens_to_get.checked_div(ido_token_decimal).unwrap(); // 10000000000000000 / 10 ^ 16 = 1

        let ido_token_price_ratio = authorized_sale_account.ido_token_price_ratio as u128;
        let _divide_by_ratio = ido_token_price_ratio
            .checked_mul(pay_token_token_decimal)
            .unwrap(); // (4 * 10 ^ 3) * 10 ^ 4 = 4 * 10 ^ 7

        let mut _amount_in_pay_token = (_ido_tokens_to_get).checked_mul(_divide_by_ratio).unwrap(); // 1 * 4 * 10 ^ 7 = 4 * 10 ^ 7
        let ido_token_price_multiplier = authorized_sale_account.ido_token_price_multiplier as u128;
        _amount_in_pay_token = _amount_in_pay_token
            .checked_div(ido_token_price_multiplier)
            .unwrap(); // (4 * 10 ^ 7) / 10 ^ 4 = 4 * 10 ^ 3 USDC tokens
        _amount_in_pay_token
    }
    /// Calculate the amount of Ido Tokens bought
    pub fn calculate_ido_tokens_bought(&self, _amount_in_pay_token: u128) -> u128 {
        let authorized_sale_account = &self.authorized_sale_account;

        let ido_token_decimal: u128 = 10u128.checked_pow(18 - 2).unwrap();
        let pay_token_token_decimal: u128 = 10u128.checked_pow(6 - 2).unwrap();

        let _amount_in_pay_token = _amount_in_pay_token
            .checked_mul(authorized_sale_account.ido_token_price_multiplier as u128)
            .unwrap(); // 250_000_000 * 10_000 = 2_500_000_000_000
        let _divide_by_ratio = (authorized_sale_account.ido_token_price_ratio as u128)
            .checked_mul(pay_token_token_decimal)
            .unwrap(); // 4_000 * 10_000 = 40_000_000
        let mut _ido_tokens_to_get = _amount_in_pay_token.checked_div(_divide_by_ratio).unwrap(); // 2_500_000_000_000 / 40_000_000 = 62_500
        _ido_tokens_to_get = _ido_tokens_to_get.checked_mul(ido_token_decimal).unwrap(); // 62_500 * 10_000_000_000_000_000 = 625_000_000_000_000_000_000
        _ido_tokens_to_get
    }
}

/// Validation struct for reading fields of both SaleAccount and AuthorizedSaleAccount
#[derive(Accounts)]
pub struct ReadAccounts<'info> {
    pub authorized_sale_account: Account<'info, AuthorizedSaleAccount>,
    pub sale_account: Account<'info, SaleAccount>,
    pub user: Signer<'info>,
}

/// Validation struct for reading buyer's info and all data fields (AuthorizedSaleAccount + SaleAccount)
#[derive(Accounts)]
#[instruction(_buyer: Pubkey)]
pub struct ReadBuyerInfoAndAccounts<'info> {
    pub user: Signer<'info>,
    #[account(seeds = [b"buyer-info", _buyer.as_ref()], bump = buyer_info.bump)]
    pub buyer_info: Account<'info, BuyerInfo>,
    pub sale_account: Account<'info, SaleAccount>,
    pub authorized_sale_account: Account<'info, AuthorizedSaleAccount>,
}
impl<'info> ReadBuyerInfoAndAccounts<'info> {
    /// Calculate the amount of Ido Tokens bought
    pub fn calculate_ido_tokens_bought(&self, _amount_in_pay_token: u128) -> u128 {
        let authorized_sale_account = &self.authorized_sale_account;

        let ido_token_decimal: u128 = 10u128.checked_pow(18 - 2).unwrap();
        let pay_token_token_decimal: u128 = 10u128.checked_pow(6 - 2).unwrap();

        let _amount_in_pay_token = _amount_in_pay_token
            .checked_mul(authorized_sale_account.ido_token_price_multiplier as u128)
            .unwrap(); // 250_000_000 * 10_000 = 2_500_000_000_000
        let _divide_by_ratio = (authorized_sale_account.ido_token_price_ratio as u128)
            .checked_mul(pay_token_token_decimal)
            .unwrap(); // 4_000 * 10_000 = 40_000_000
        let mut _ido_tokens_to_get = _amount_in_pay_token.checked_div(_divide_by_ratio).unwrap(); // 2_500_000_000_000 / 40_000_000 = 62_500
        _ido_tokens_to_get = _ido_tokens_to_get.checked_mul(ido_token_decimal).unwrap(); // 62_500 * 10_000_000_000_000_000 = 625_000_000_000_000_000_000
        _ido_tokens_to_get
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
