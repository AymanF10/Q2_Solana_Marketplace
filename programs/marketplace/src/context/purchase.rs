use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
use anchor_spl::{associated_token::AssociatedToken, metadata::{MasterEditionAccount, Metadata, MetadataAccount}, token::{close_account, mint_to, transfer_checked, CloseAccount, MintTo, TransferChecked}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::state::{Listing, Marketplace};

#[derive(Accounts)]
pub struct Purchase<'info>{
    #[account(mut)]
    pub taker: Signer<'info>, 
    #[account(mut)]
    pub maker: SystemAccount<'info>, 

    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    marketplace: Account<'info, Marketplace>, 

    maker_mint: InterfaceAccount<'info, Mint>, 

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = maker_mint,
        associated_token::authority = taker,
    )]
    pub taker_ata: InterfaceAccount<'info, TokenAccount>, 

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = rewards_mint,
        associated_token::authority = taker,
    )]
    pub taker_ata_reward: InterfaceAccount<'info, TokenAccount>, 

    #[account(
        mut,
        close = maker, 
        seeds = [marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump = listing.bump,
    )]
    listing: Account<'info, Listing>, 

    #[account(
        mut,
        associated_token::mint = maker_mint,
        associated_token::authority = listing,
    )]
    vault: InterfaceAccount<'info, TokenAccount>, 

    #[account(
        mut,
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump = marketplace.treasury_bump,
    )]
    pub treasury: SystemAccount<'info>, 

    #[account(
        mut,
        seeds = [b"rewards", marketplace.key().as_ref()],
        bump = marketplace.rewards_bump,
        mint::decimals = 6,
        mint::authority = marketplace,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    

    pub associated_token_program: Program<'info, AssociatedToken>, 
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>, 
}

impl <'info> Purchase<'info>{
    pub fn send_sol(&mut self) ->Result<()>{
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts= Transfer{
            from: self.taker.to_account_info(),
            to: self.maker.to_account_info(), 
        };

        let cpi_ctx = CpiContext::new(cpi_program.clone(), cpi_accounts);

       
        let fee = self.marketplace.fee as u64;
        let amount = self.listing.price.checked_sub(fee).unwrap();

       
        transfer(cpi_ctx, amount)?;

        let cpi_accounts= Transfer{
            from: self.taker.to_account_info(), 
            to: self.treasury.to_account_info(), 
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

       
        transfer(cpi_ctx, fee)?;
        Ok(())
    }

    pub fn receive_nft(&mut self)-> Result<()>{
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked{
            from: self.vault.to_account_info(), 
            mint: self.maker_mint.to_account_info(), 
            to: self.taker_ata.to_account_info(), 
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

    pub fn receive_rewards(&mut self) ->Result<()>{
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo{
            mint: self.rewards_mint.to_account_info(), 
            to: self.taker_ata_reward.to_account_info(),
            authority: self.marketplace.to_account_info(), 
        };

      
        let seeds = &[
            b"marketplace",
            self.marketplace.name.as_str().as_bytes().as_ref(),
            &[self.marketplace.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

   
        mint_to(cpi_ctx, 1)?;

        Ok(())
    }

    pub fn close_mint_vault(&mut self)->Result<()>{
        
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

        
        close_account(cpi_ctx)?;

        Ok(())
    }
}