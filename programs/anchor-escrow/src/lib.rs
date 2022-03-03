pub mod utils;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

declare_id!("EL7sdb92YQFzXdU9hgxHd67znWXCSiLaergfFr2ZdhsZ");

#[program]
pub mod anchor_escrow {
    use super::*;

    const ESCROW_PDA_SEED: &[u8] = b"escrow";

    pub fn initialize(
        ctx: Context<Initialize>, nonce: u8
    ) -> ProgramResult {
        msg!("initialize");

        let spin_win = &mut ctx.accounts.spin_win;
        spin_win.token_vault = ctx.accounts.token_vault.key();
        spin_win.nonce = nonce;

        Ok(())
    }

    pub fn add_item(
        ctx: Context<AddItem>,
        ratio: u8,
        reward_type: u8,
        amount: u64,
    ) -> Result<()> {
        msg!("add_item");

        let spin_win = &mut ctx.accounts.spin_win;
        let item_list = &mut spin_win.item_list;
        let item = SpinItem {ratio: ratio, reward_type: reward_type, amount: amount, nft_account: ctx.accounts.nft_account, reward_account: ctx.accounts.reward_account};
        item_list.push(item);

        if reward_type == 0 {
            // reward token
            spl_token_transfer_without_seed(
                TokenTransferParamsWithoutSeed{
                    source : spin_item.reward_account.clone(),
                    destination : spin_win.token_vault.to_account_info(),
                    authority : ctx.accounts.owner.clone,
                    token_program : ctx.accounts.token_program.clone(),
                    amount : spin_item.amount,
                }
            )?;
        } else {
            // nft token
            spl_token_transfer_without_seed(
                TokenTransferParamsWithoutSeed{
                    source : spin_item.nft_account.clone(),
                    destination : spin_win.token_vault.to_account_info(),
                    authority : ctx.accounts.owner.clone,
                    token_program : ctx.accounts.token_program.clone(),
                    amount : 1,
                }
            )?;
        }

        Ok(())
    }

    // pub fn set_item(
    //     ctx: Context<SpinWin>,
    //     index: u8,
    //     ratio: u8,
    //     reward_type: u8,
    //     amount: u64,
    //     nft_account : Pubkey,
    //     reward_account: Pubkey,
    // ) -> Result<()> {
    //     let item_list = &mut ctx.accounts.item_list;
    //     if index < item_list.len() {
    //         let item = &mut item_list[index];
    //         item.ratio = ratio;
    //         item.reward_type = reward_type;
    //         item.amount = amount;
    //         item.nft_account = nft_account;
    //         item.reward_account = reward_account;
    //     }

    //     Ok(())
    // }

    fn get_spinresult(ctx: Context<SpinWheel>) -> u8 {
        let spin_win = &mut ctx.accounts.spin_win;
        let item_list = &mut spin_win.item_list;
        let c = clock::Clock::get().unwrap();
        let r = c.unix_timestamp % 100;
        let start = 0;
        for (pos, item) in item_list.iter().enumerate() {
            let end = start + item.ratio;
            if r >= start && r < end {
                return pos;
            }
            start = end;
        }

        return 0;
    }

    pub fn spin_wheel(ctx: Context<SpinWheel>) -> Result<()> {

        let spin_win = &mut ctx.accounts.spin_win;
        let spin_index = get_spinresult(ctx);
        let spin_item = &mut spin_win.item_list[spin_index];

        if spin_item.reward_type == 0 {
            // reward token
            spl_token_transfer_without_seed(
                TokenTransferParamsWithoutSeed{
                    source : spin_item.reward_account.clone(),
                    destination : ctx.accounts.dest_account.clone(),
                    authority : spin_win.to_account_info(),
                    token_program : ctx.accounts.token_program.clone(),
                    amount : spin_item.amount,
                }
            )?;
        } else {
            // nft token
            spl_token_transfer_without_seed(
                TokenTransferParamsWithoutSeed{
                    source : spin_item.nft_account.clone(),
                    destination : ctx.accounts.dest_account.clone(),
                    authority : spin_win.to_account_info(),
                    token_program : ctx.accounts.token_program.clone(),
                    amount : 1,
                }
            )?;
        }

        Ok(())
    }
}



///////////////////////////////////////////////////////////////////////

#[derive(Accounts)]
pub struct SpinItem<'info> {
    ratio: u8, // percent
    reward_type: u8, // 0 : token, 1 : nft
    amount: u64, // reward amount

    #[account(mut)]
    nft_account: AccountInfo<'info>,

    #[account(mut)]
    reward_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SpinWin {
    pub item_list: Vec<SpinItem>,
    pub token_vault: Account<'info, TokenAccount>,
    pub nonce: u8,

    #[account(mut)]
    source_account : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SpinWheel<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account()]
    spin_win : ProgramAccount<'info, SpinWin>,

    #[account(mut,owner=spl_token::id())]
    dest_account : AccountInfo<'info>,

    #[account(mut,owner=spl_token::id())]
    token_program : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AddItem<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>, 

    #[account(mut)]
    spin_win : ProgramAccount<'info, SpinWin>,

    #[account(mut,owner=spl_token::id())]
    nft_account : AccountInfo<'info>,

    #[account(mut,owner=spl_token::id())]
    reward_account : AccountInfo<'info>,

    #[account(address=spl_token::id())]
    token_program : AccountInfo<'info>,

    system_program : Program<'info,System>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub spin_win: ProgramAccount<'info, SpinWin>,

    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,
}