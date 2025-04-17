use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{close_account, transfer_checked, CloseAccount, TransferChecked}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::state::{listing, Listing, Marketplace};

#[derive(Accounts)]
pub struct Delist<'info>{
    #[account(mut)]
    pub maker: Signer<'info>, 
    pub maker_mint: InterfaceAccount<'info, Mint>, 

    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>, 

    #[account(
        mut,
        associated_token::mint = maker_mint,
        associated_token::authority = maker,
    )]
    pub maker_ata: InterfaceAccount<'info, TokenAccount>, 

    #[account(
        mut,
        associated_token::mint = maker_mint,
        associated_token::authority = listing,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>, 
    
    #[account(
        mut,
        close = maker, 
        seeds = [marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump = listing.bump,
    )]
    pub listing: Account<'info, Listing>, 

    pub system_program: Program<'info, System>, 
    pub token_program: Interface<'info, TokenInterface>, 
    pub associated_token_program: Program<'info, AssociatedToken>, 
}

impl <'info> Delist<'info> {
    pub fn delist(&mut self) ->Result<()>{
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked{
            from: self.vault.to_account_info(), 
            mint: self.maker_mint.to_account_info(), 
            to: self.maker_ata.to_account_info(), 
            authority: self.listing.to_account_info(), 
        };

        let seeds = &[
            &self.marketplace.key().to_bytes()[..],
            &self.maker_mint.key().to_bytes()[..],
            &[self.listing.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        
        transfer_checked(cpi_ctx, self.vault.amount, self.maker_mint.decimals)?;

        Ok(())
    }

    pub fn close_mint_vault(&mut self)->Result<()>{
        // Create signer seeds 
        let seeds = &[
            &self.marketplace.key().to_bytes()[..], 
            &self.maker_mint.key().to_bytes()[..],
            &[self.listing.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = CloseAccount{
            account: self.vault.to_account_info(), 
            destination: self.maker.to_account_info(), 
            authority: self.listing.to_account_info(), 
        };

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // Close the vault 
        close_account(cpi_ctx)?;

        Ok(())
    }
}