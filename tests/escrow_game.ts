import * as anchor from "@coral-xyz/anchor";
import { AnchorError, Program } from "@coral-xyz/anchor";
import { EscrowGame } from "../target/types/escrow_game";
import {createMint, getAccount, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo} from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";
import { randomBytes } from "crypto";
import { assert } from "chai";

describe("Escrow game", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.EscrowGame as Program<EscrowGame>;
  const playerOne = provider.wallet as anchor.Wallet;
  const playerTwo = Keypair.generate();
  const oracle_mock = Keypair.generate();

  let seed = new anchor.BN(randomBytes(8));
  let escrow = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), seed.toArrayLike(Buffer, "le", 8)],
    program.programId
  )[0];

  let mintAddress: PublicKey;
  let escrowTokenAccount: PublicKey;
  let playerOneTokenAccount: PublicKey;
  let playerTwoTokenAccount: PublicKey;

  const initialBalance = 100000 * 10 ** 6;
  const fee = 1000 * 10 ** 6;

  const ethPrice = 262668321338;
  const ethExponent = -8;

  before(async () => {
    await InitializeToken();
  });

  it("Should initialize escrow account", async () => {
    const entry_fee = new anchor.BN(fee);
    const tx = await program.methods
      .initialize(seed, entry_fee)
      .accounts({
        player: playerOne.publicKey,
        escrow: escrow,
      })
      .signers([playerOne.payer]).rpc();

    const escrowData = await program.account.escrow.fetch(escrow);
    assert.equal(escrowData.entryFee.toNumber, entry_fee.toNumber);
  });

  it("Should initialize game", async () => {
    await setEthPrice(ethPrice, ethExponent);

    const is_leg_up = false;
    const transactionSignature = await program.methods
      .initializeGame(is_leg_up)
      .accounts({
        player: playerOne.publicKey,
        usdcMint: mintAddress,
        playerTokenAccount: playerOneTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        escrow: escrow,
        oracleMock: oracle_mock.publicKey,
      }).rpc();

    let balance = await getTokenBalance(playerOneTokenAccount);
    assert.equal(
      balance.toString(),
      String(initialBalance - fee),
      "Account balance not correct"
    );

    balance = await getTokenBalance(escrowTokenAccount);
    assert.equal(
      balance.toString(),
      String(fee),
      "Account balance not correct"
    );

    const escrowData = await program.account.escrow.fetch(escrow);
    assert.equal(
      escrowData.ethPrice.toNumber,
      new anchor.BN(ethPrice).toNumber
    );
    assert.equal(escrowData.ethExponent, ethExponent);
    assert.equal(
      String(escrowData.status),
      String({ initialize: {} }),
      "State not correct"
    );
  });

  it("Should accept a game if variation is not bigger than 1%", async () => {
    await setEthPrice(ethPrice * 1.005, ethExponent);
    const transactionSignature = await program.methods
      .acceptGame()
      .accounts({
        player: playerTwo.publicKey,
        usdcMint: mintAddress,
        playerTokenAccount: playerTwoTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        escrow: escrow,
        oracleMock: oracle_mock.publicKey,
      })
      .signers([playerTwo]).rpc();

    let balance = await getTokenBalance(playerTwoTokenAccount);
    assert.equal(
      balance.toString(),
      String(initialBalance - fee),
      "Account balance not correct"
    );

    balance = await getTokenBalance(escrowTokenAccount);
    assert.equal(
      balance.toString(),
      String(fee * 2),
      "Account balance not correct"
    );
    const escrowData = await program.account.escrow.fetch(escrow);
    assert.equal(
      String(escrowData.status),
      String({ accepted: {} }),
      "State not correct"
    );
  });

  it("Should close the game if difference is bigger than 5%", async () => {
    await setEthPrice(ethPrice * 1.051, ethExponent);

    const transactionSignature = await program.methods
      .closeGame()
      .accounts({
        player: playerTwo.publicKey,
        usdcMint: mintAddress,
        playerTokenAccount: playerTwoTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        escrow: escrow,
        oracleMock: oracle_mock.publicKey,
      })
      .signers([playerTwo]).rpc();

    let balance = await getTokenBalance(playerTwoTokenAccount);
    assert.equal(
      balance.toString(),
      String(initialBalance + fee),
      "Account balance not correct"
    );

    balance = await getTokenBalance(escrowTokenAccount);
    assert.equal(balance.toString(), String(0), "Account balance not correct");
    const escrowData = await program.account.escrow.fetch(escrow);
    assert.equal(
      String(escrowData.status),
      String({ closed: {} }),
      "State not correct"
    );
  });

  it("Should close an escrow account after the game has ended", async () => {
    const transactionSignature = await program.methods
      .withdrawGame()
      .accounts({
        player: playerOne.publicKey,
        usdcMint: mintAddress,
        playerTokenAccount: playerOneTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        escrow: escrow,
      }).rpc();

    try {
      const escrowData = await program.account.escrow.fetch(escrow);
    } catch (_err) {
      const err = _err as Error;
      const errMsg = "Account does not exist";
      assert.equal(
        err.message.includes(errMsg),
        true,
        "Error message does not match expected text."
      );
    }
  });

  it("Should withdraw a game before it is accepted", async () => {
    const [escrow, escrowTokenAccount] = await initializeEscrow();

    // Withdraw game
    const transactionSignature = await program.methods
      .withdrawGame()
      .accounts({
        player: playerOne.publicKey,
        usdcMint: mintAddress,
        playerTokenAccount: playerOneTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        escrow: escrow,
      }).rpc();

    let balance = await getTokenBalance(playerOneTokenAccount);
    assert.equal(
      balance.toString(),
      String(initialBalance),
      "Account balance not correct"
    );

    try {
      const escrowData = await program.account.escrow.fetch(escrow);
    } catch (_err) {
      const err = _err as Error;
      const errMsg = "Account does not exist";
      assert.equal(
        err.message.includes(errMsg),
        true,
        "Error message does not match expected text."
      );
    }
  });

  it("Should not withdraw a game if not the creator", async () => {
    const [escrow, escrowTokenAccount] = await initializeEscrow();

    try {
      // Withdraw game
      const transactionSignature = await program.methods
        .withdrawGame()
        .accounts({
          player: playerTwo.publicKey,
          usdcMint: mintAddress,
          playerTokenAccount: playerTwoTokenAccount,
          escrowTokenAccount: escrowTokenAccount,
          escrow: escrow,
        })
        .signers([playerTwo]).rpc();
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      const errMsg = "Not creator of escrow";
      assert.equal(err.error.errorMessage, errMsg, "Incorrect error");
    }
  });

  it("Should not withdraw a game if already accepted", async () => {
    const [escrow, escrowTokenAccount] = await initializeEscrow();

    try {
      // Withdraw game
      const transactionSignature = await program.methods
        .withdrawGame()
        .accounts({
          player: playerOne.publicKey,
          usdcMint: mintAddress,
          playerTokenAccount: playerOneTokenAccount,
          escrowTokenAccount: escrowTokenAccount,
          escrow: escrow,
        }).rpc();
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      const errMsg = "Escrow deal not available";
      assert.equal(err.error.errorMessage, errMsg, "Incorrect error");
    }
  });

  it("Should not accept a game already accepted", async () => {
    const [escrow, escrowTokenAccount] = await initializeEscrow();

    const transactionSignature = await program.methods
      .acceptGame()
      .accounts({
        player: playerTwo.publicKey,
        usdcMint: mintAddress,
        playerTokenAccount: playerTwoTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        escrow: escrow,
        oracleMock: oracle_mock.publicKey,
      })
      .signers([playerTwo]).rpc();

    const escrowData = await program.account.escrow.fetch(escrow);
    assert.equal(
      String(escrowData.status),
      String({ accepted: {} }),
      "State not correct"
    );

    try {
      const transactionSignature = await program.methods
        .acceptGame()
        .accounts({
          player: playerTwo.publicKey,
          usdcMint: mintAddress,
          playerTokenAccount: playerTwoTokenAccount,
          escrowTokenAccount: escrowTokenAccount,
          escrow: escrow,
          oracleMock: oracle_mock.publicKey,
        })
        .signers([playerTwo]).rpc();
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      const errMsg = "Escrow deal not available";
      assert.equal(err.error.errorMessage, errMsg, "Incorrect error");
    }
  });

  it("Should not accept a game if variation is bigger 1%", async () => {
    const [escrow, escrowTokenAccount] = await initializeEscrow();

    await setEthPrice(ethPrice * 1.03, ethExponent);
    try {
      const transactionSignature = await program.methods
        .acceptGame()
        .accounts({
          player: playerTwo.publicKey,
          usdcMint: mintAddress,
          playerTokenAccount: playerTwoTokenAccount,
          escrowTokenAccount: escrowTokenAccount,
          escrow: escrow,
          oracleMock: oracle_mock.publicKey,
        })
        .signers([playerTwo]).rpc();
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      const errMsg = "Cannot join game";
      assert.equal(err.error.errorMessage, errMsg, "Incorrect error");
    }
  });

  it("Should not close a game if price difference is not over 5%", async () => {
    const [escrow, escrowTokenAccount] = await initializeEscrow();

    const transactionSignature = await program.methods
      .acceptGame()
      .accounts({
        player: playerTwo.publicKey,
        usdcMint: mintAddress,
        playerTokenAccount: playerTwoTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        escrow: escrow,
        oracleMock: oracle_mock.publicKey,
      })
      .signers([playerTwo]).rpc();

    const escrowData = await program.account.escrow.fetch(escrow);
    assert.equal(
      String(escrowData.status),
      String({ accepted: {} }),
      "State not correct"
    );

    await setEthPrice(ethPrice * 1.03, ethExponent);
    try {
      const transactionSignature = await program.methods
        .closeGame()
        .accounts({
          player: playerOne.publicKey,
          usdcMint: mintAddress,
          playerTokenAccount: playerOneTokenAccount,
          escrowTokenAccount: escrowTokenAccount,
          escrow: escrow,
          oracleMock: oracle_mock.publicKey,
        })
        .signers([playerOne.payer]).rpc();
    } catch (_err) {
      assert.isTrue(_err instanceof AnchorError);
      const err: AnchorError = _err;
      const errMsg = "No escrow winner yet";
      assert.equal(err.error.errorMessage, errMsg, "Incorrect error");
    }
  });

  it("Should be possible to win leg up", async () => {
    const [escrow, escrowTokenAccount] = await initializeEscrow();

    let transactionSignature = await program.methods
      .acceptGame()
      .accounts({
        player: playerTwo.publicKey,
        usdcMint: mintAddress,
        playerTokenAccount: playerTwoTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        escrow: escrow,
        oracleMock: oracle_mock.publicKey,
      })
      .signers([playerTwo]).rpc();

    await setEthPrice(ethPrice * 1.051, ethExponent);
    transactionSignature = await program.methods
      .closeGame()
      .accounts({
        player: playerOne.publicKey,
        usdcMint: mintAddress,
        playerTokenAccount: playerOneTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        escrow: escrow,
        oracleMock: oracle_mock.publicKey,
      })
      .signers([playerOne.payer]).rpc();

    let balance = await getTokenBalance(playerOneTokenAccount);
    assert.equal(
      balance.toString(),
      String(initialBalance + fee),
      "Account balance not correct"
    );

    balance = await getTokenBalance(escrowTokenAccount);
    assert.equal(balance.toString(), String(0), "Account balance not correct");
    const escrowData = await program.account.escrow.fetch(escrow);
    assert.equal(
      String(escrowData.status),
      String({ closed: {} }),
      "State not correct"
    );
  });

  // ----------------------------- Helper functions -----------------------------
  async function initializeEscrow(): Promise<[PublicKey, PublicKey]> {
    await InitializeToken();
    seed = new anchor.BN(randomBytes(8));
    escrow = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    )[0];
    escrowTokenAccount = getAssociatedTokenAddressSync(
      mintAddress,
      escrow,
      true
    );

    await setEthPrice(ethPrice, ethExponent);
    const entry_fee = new anchor.BN(fee);
    const is_leg_up = true;

    await program.methods
      .initialize(seed, entry_fee)
      .accounts({
        player: playerOne.publicKey,
        escrow: escrow,
      })
      .signers([playerOne.payer]).rpc();

    let transactionSignature = await program.methods
      .initializeGame(is_leg_up)
      .accounts({
        player: playerOne.publicKey,
        usdcMint: mintAddress,
        playerTokenAccount: playerOneTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
        escrow: escrow,
        oracleMock: oracle_mock.publicKey,
      }).rpc();

    let balance = await getTokenBalance(playerOneTokenAccount);
    assert.equal(
      balance.toString(),
      String(initialBalance - fee),
      "Account balance not correct"
    );

    const escrowData = await program.account.escrow.fetch(escrow);
    assert.equal(
      String(escrowData.status),
      String({ initialize: {} }),
      "State not correct"
    );

    return [escrow, escrowTokenAccount];
  }

  async function getTokenBalance(tokenAccountPubkey) {
    const tokenAccount = await getAccount(
      provider.connection,
      tokenAccountPubkey
    );
    return tokenAccount.amount; // Amount is in base units (smallest denomination, like lamports for SOL)
  }

  async function setEthPrice(price, exponent) {
    const eth_price = new anchor.BN(price);
    const eth_exponent = exponent;
    const tx_price = await program.methods
      .setEthPrice(eth_price, eth_exponent)
      .accounts({
        payer: playerOne.publicKey,
        oracleMock: oracle_mock.publicKey,
      })
      .signers([oracle_mock]).rpc();
  }

  async function InitializeToken() {
    mintAddress = await createMint(
      provider.connection,
      playerOne.payer,
      playerOne.publicKey,
      null,
      6
    );

    escrowTokenAccount = getAssociatedTokenAddressSync(
      mintAddress,
      escrow,
      true
    );

    playerOneTokenAccount = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        playerOne.payer,
        mintAddress,
        playerOne.publicKey
      )
    ).address;

    playerTwoTokenAccount = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        playerOne.payer,
        mintAddress,
        playerTwo.publicKey
      )
    ).address;

    await mintTo(
      provider.connection,
      playerOne.payer,
      mintAddress,
      playerOneTokenAccount,
      playerOne.publicKey,
      initialBalance
    );
    await mintTo(
      provider.connection,
      playerOne.payer,
      mintAddress,
      playerTwoTokenAccount,
      playerOne.publicKey,
      initialBalance
    );
  }
});
