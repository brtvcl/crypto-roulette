import assert from 'assert';
import * as anchor from '@project-serum/anchor';
import { web3, AnchorProvider } from '@project-serum/anchor';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
const SystemProgram = web3.SystemProgram;

describe('roulette', () => {
  const provider = AnchorProvider.local();
  anchor.setProvider(provider);
  const tableWallet = anchor.web3.Keypair.generate();

  const program = anchor.workspace.Roulette;

  let betters = new Array(13).fill(null);
  before(async function () {
    const signature = await provider.connection.requestAirdrop(tableWallet.publicKey, 100 * LAMPORTS_PER_SOL);
    const latestBlockHash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: signature,
    });

    betters = betters.map(async () => {
      const better = anchor.web3.Keypair.generate();
      const signature = await provider.connection.requestAirdrop(better.publicKey, 5 * LAMPORTS_PER_SOL);
      const latestBlockHash = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction({
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        signature: signature,
      });

      return better;
    });

    const tableBalance = await provider.connection.getBalance(tableWallet.publicKey);

    console.log(tableBalance / LAMPORTS_PER_SOL);

    betters = await Promise.all(betters);
  });

  it('Creates a table', async () => {
    const result = await program.rpc.create('QWER123', {
      accounts: {
        table: tableWallet.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [tableWallet],
    });

    const table = await program.account.table.fetch(tableWallet.publicKey);

    // assert.equal(table.amountAccumulated.toNumber(), new anchor.BN(0).toNumber());
    assert.equal(table.id, 'QWER123');
  });

  it('Accepts bets', async () => {
    let available_positions = [
      'RED',
      'BLACK',
      'GREEN',
      'EVEN',
      'ODD',
      '1-18',
      '19-36',
      '1-12',
      '13-24',
      '25-36',
      'COL1',
      'COL2',
      'COL3',
    ];

    const plays = await Promise.all(
      available_positions.map(async (BET, i) => {
        const better = betters[i];
        const balanceBefore = await provider.connection.getBalance(better.publicKey);

        await program.rpc.bet(BET, new anchor.BN(2 * LAMPORTS_PER_SOL), {
          accounts: {
            table: tableWallet.publicKey,
            user: better.publicKey,
            systemProgram: SystemProgram.programId,
          },
          signers: [better],
        });

        const balanceAfter = await provider.connection.getBalance(better.publicKey);

        return {
          balanceBefore,
          balanceAfter,
          BET,
        };
      })
    );

    const table = await program.account.table.fetch(tableWallet.publicKey);

    assert.equal(table.positions.length, available_positions.length);

    assert.equal(
      plays.every((p) => p.balanceBefore / LAMPORTS_PER_SOL == 5),
      plays.every((p) => p.balanceAfter / LAMPORTS_PER_SOL == 3)
    );
  });

  it('Doesnt accepts wrong bets', async () => {
    let isError = false;
    try {
      await program.rpc.bet('BLUE', new anchor.BN(2 * LAMPORTS_PER_SOL), {
        accounts: {
          table: tableWallet.publicKey,
          user: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        },
      });
    } catch (error) {
      isError = true;
    }
    assert.ok(isError);
  });

  it('Spins', async () => {
    await program.rpc.spin({
      accounts: {
        table: tableWallet.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
    });

    const table = await program.account.table.fetch(tableWallet.publicKey);

    assert.ok(table.result != -1);
  });

  it('Claiming prizes', async () => {
    const tableBefore = await program.account.table.fetch(tableWallet.publicKey);

    const result = tableBefore.result;
    const reds = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];
    const blacks = [2, 4, 6, 8, 10, 11, 13, 15, 17, 20, 22, 24, 26, 28, 29, 31, 33, 35];

    let is_green = result == 0;
    let wins = {
      GREEN: is_green,
      RED: reds.includes(result),
      BLACK: blacks.includes(result),
      EVEN: !is_green && result % 2 == 0,
      ODD: result % 2 == 1,
      '1-18': result >= 1 && result <= 18,
      '19-36': result >= 19,
      '1-12': result >= 1 && result <= 12,
      '13-24': result >= 13 && result <= 24,
      '25-36': result >= 25,
      COL1: result % 3 == 1,
      COL2: result % 3 == 2,
      COL3: !is_green && result % 3 == 0,
    };

    const claims = await Promise.all(
      betters.map(async (better) => {
        const balanceBefore = await provider.connection.getBalance(better.publicKey);
        const BET = tableBefore.positions.find((p) => {
          return better.publicKey.toBase58() == p.userAddress.toBase58();
        })?.position;

        const hasWon = wins[BET];

        await program.rpc.claimPrize({
          accounts: {
            table: tableWallet.publicKey,
            user: better.publicKey,
            systemProgram: SystemProgram.programId,
          },
          signers: [better],
        });

        const balanceAfter = await provider.connection.getBalance(better.publicKey);

        return {
          balanceBefore: balanceBefore / LAMPORTS_PER_SOL,
          balanceAfter: balanceAfter / LAMPORTS_PER_SOL,
          hasWon,
          BET,
        };
      })
    );

    const tableAfter = await program.account.table.fetch(tableWallet.publicKey);

    assert.ok(tableBefore.positions.every((p) => p.isClaimed == false));
    assert.ok(tableAfter.positions.every((p) => p.isClaimed == true));
    assert.ok(claims.every((c) => (c.hasWon ? c.balanceAfter > c.balanceBefore : c.balanceAfter == c.balanceBefore)));
  });
});
