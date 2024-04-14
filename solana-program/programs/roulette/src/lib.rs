use anchor_lang::prelude::*;

declare_id!("FWnbqf2rUMkLjxUiWtBKhgSKfsqqZSrxjmifBhWPQieQ");

#[program]
pub mod roulette {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
