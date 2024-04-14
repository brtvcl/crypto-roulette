use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("65n1nbeq99EoGUje3bLAiZDgY6WNFCZ4AicEY1WdGoyB");



#[program]
pub mod roulette {
    use super::*;

    pub fn create(ctx: Context<Create>, id: String) -> Result<()> {
        let table = &mut ctx.accounts.table;
        table.id = id;
        table.result = -1;
        Ok(())
    } 

    pub fn bet(ctx: Context<Bet>, position: String, amount: u64) -> ProgramResult {
        let available_positions = [
            "RED",
            "BLACK",
            "GREEN",
            "EVEN",
            "ODD",
            "1-18",
            "19-36",
            "1-12",
            "13-24",
            "25-36",
            "COL3",
            "COL1",
            "COL2"
        ];
        if !available_positions.contains(&(position.as_str())) {
            return Err(ProgramError::InvalidArgument);
        }

        let table = &mut ctx.accounts.table;

        if (table.result !=-1) {
            return Err(ProgramError::InvalidArgument);
        }

        if table.positions.len() >= 13 {
            return Err(ProgramError::InvalidArgument);
        }

        

        let position = PositionStruct {
            position: position.to_string(),
            user_address: ctx.accounts.user.key(),
            amount: amount,
            is_claimed: false
        };
        
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.table.key(),
            amount
        );

        let _ = anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.table.to_account_info()
            ]
        );

        (&mut ctx.accounts.table).positions.push(position);

        // TODO: If acculumated enough, spin
        Ok(())
    }

    pub fn spin(ctx: Context<Spin>) -> Result<()> {
        let table = &mut ctx.accounts.table;
               

        let clock = Clock::get()?;
        let num = (clock.unix_timestamp % 37) as i8;
        table.result = num; // Random integer between 0-36 (inclusive)

        
        Ok(())
    }

    pub fn claim_prize(ctx: Context<ClaimPrize>) -> ProgramResult {
        let table = &mut ctx.accounts.table;
        let user = &mut ctx.accounts.user;

        const REDS: [i8; 18] = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];
        const BLACKS: [i8; 18] = [2, 4, 6, 8, 10, 11, 13, 15, 17, 20, 22, 24, 26, 28, 29, 31, 33, 35];

        let num = table.result;

        let is_green: bool = num == 0;
        let is_red: bool = REDS.contains(&num);
        let is_black: bool = BLACKS.contains(&num);
        let is_even: bool = !is_green && num % 2 == 0;
        let is_odd: bool = num % 2 == 1;
        let is_1_18: bool = num >= 1 && num <=18;
        let is_19_36: bool = num >= 19;
        let is_1_12: bool = num >= 1 && num <= 12;
        let is_13_24: bool = num >= 13 && num <= 24;
        let is_25_36: bool = num >=25;
        let is_col3: bool = !is_green && num % 3 == 0;
        let is_col1: bool = num % 3 == 1;
        let is_col2: bool = num % 3 == 2;


        let mut position = "NONE";
        let mut amount = 0;
        let mut transfer: u64 = 0;
        for p in &mut table.positions {
            if p.user_address == user.key() {
                if p.is_claimed {
                    return Err(ProgramError::IllegalOwner);
                }
                position = &p.position;
                amount = p.amount;
                p.is_claimed = true;
            }
        }

        if position == "NONE" {
            return Err(ProgramError::IllegalOwner);
        }
        
        if position == "BLACK" && is_black {
            transfer = amount*2;
        }
        if position == "RED" && is_red {
            transfer = amount*2;
        }
        if position == "GREEN" && is_green {
            transfer = amount*36;
        }
        if position == "EVEN" && is_even {
            transfer = amount*2;
        }
        if position == "ODD" && is_odd {
            transfer = amount*2;
        }
        if position == "1-18" && is_1_18 {
            transfer = amount*2;
        }
        if position == "19-36" && is_19_36 {
            transfer = amount*2;
        }
        if position == "1-12" && is_1_12 {
            transfer = amount*3;
        }
        if position == "13-24" && is_13_24 {
            transfer = amount*3;
        }
        if position == "25-36" && is_25_36 {
            transfer = amount*3;
        }
        if position == "COL3" && is_col3 {
            transfer = amount*3;
        }
        if position == "COL1" && is_col1 {
            transfer = amount*3;
        }
        if position == "COL2" && is_col2 {
            transfer = amount*3;
        }

        **table.to_account_info().try_borrow_mut_lamports()? -= transfer;
        **user.to_account_info().try_borrow_mut_lamports()? += transfer;

        Ok(())
    }
    
}


#[derive(Accounts)]
pub struct Create<'info> {
    #[account(init, payer=user, space=1500)]
    pub table : Account<'info, Table>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[account]
pub struct Table {
    pub id: String,
    pub positions: Vec<PositionStruct>,
    pub result: i8
}

#[derive(Accounts)]
pub struct Bet<'info> {
    #[account(mut)]
    pub table: Account<'info, Table>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Spin<'info> {
    #[account(mut)]
    pub table: Account<'info, Table>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    pub table: Account<'info, Table>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct PositionStruct {
    pub position: String,
    pub amount: u64,
    pub user_address: Pubkey,
    pub is_claimed: bool
}