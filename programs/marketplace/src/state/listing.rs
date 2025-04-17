use anchor_lang::prelude::*;

#[account]
pub struct Listing{
    pub maker: Pubkey, 
    pub maker_mint: Pubkey,
    pub bump: u8,
    pub price: u64, 
}

impl Space for Listing {
    
    const INIT_SPACE: usize = 8 + 32 + 32 + 1 + 8;
}

