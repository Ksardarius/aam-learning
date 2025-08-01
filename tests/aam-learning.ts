import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { AamLearning } from "../target/types/aam_learning";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  getMint,
  Mint,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { expect } from "chai";

describe("full-pipeline", () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.aamLearning as Program<AamLearning>;

  const tokenAMintKeypair = anchor.web3.Keypair.generate();
  const tokenBMintKeypair = anchor.web3.Keypair.generate();
  const lpTokenMintKeypair = anchor.web3.Keypair.generate();

  const liquidityProviderKeypair = anchor.web3.Keypair.generate();
  const systemUser1Keypair = anchor.web3.Keypair.generate();

  let poolStatePda: anchor.web3.PublicKey;
  const poolTokenAVaultKeypair = anchor.web3.Keypair.generate();
  const poolTokenBVaultKeypair = anchor.web3.Keypair.generate();

  it("should create mints", async () => {
    const mintAPublicKey = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      6,
      tokenAMintKeypair
    );

    const mintBPublicKey = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      6,
      tokenBMintKeypair
    );

    console.log("Mint A tx", mintAPublicKey.toBase58());
    console.log("Mint B tx", mintBPublicKey.toBase58());

    const mintA = await getMint(
      provider.connection,
      tokenAMintKeypair.publicKey
    );
    const mintB = await getMint(
      provider.connection,
      tokenBMintKeypair.publicKey
    );

    expect(mintA.address.toBase58()).to.equal(
      tokenAMintKeypair.publicKey.toBase58()
    );
    expect(mintB.address.toBase58()).to.equal(
      tokenBMintKeypair.publicKey.toBase58()
    );
  });

  it("should initialize exchange", async () => {
    const tx = await program.methods
      .initializePool(30)
      .accounts({
        tokenAMint: tokenAMintKeypair.publicKey,
        tokenBMint: tokenBMintKeypair.publicKey,
        lpTokenMint: lpTokenMintKeypair.publicKey,
        payer: provider.wallet.publicKey,
        poolTokenAVault: poolTokenAVaultKeypair.publicKey,
        poolTokenBVault: poolTokenBVaultKeypair.publicKey,
      })
      .signers([
        lpTokenMintKeypair,
        poolTokenAVaultKeypair,
        poolTokenBVaultKeypair,
      ])
      .rpc();
    console.log("Your init transaction signature", tx);

    const [_poolStatePda, poolStateBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("pool_state"),
          tokenAMintKeypair.publicKey.toBuffer(),
          tokenBMintKeypair.publicKey.toBuffer(),
        ],
        program.programId
      );

    poolStatePda = _poolStatePda;

    const poolState = await program.account.poolState.fetch(poolStatePda);

    expect(poolState.tradingFees).to.equals(30);
  });

  it("should create liquidity provider with tokens", async () => {
    // create account
    await provider.connection.requestAirdrop(
      liquidityProviderKeypair.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 5
    );

    console.log(
      "Liquidity provider.",
      liquidityProviderKeypair.publicKey.toBase58()
    );

    // create token accounts
    const liquidityProviderATokenAcount = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      tokenAMintKeypair.publicKey,
      liquidityProviderKeypair.publicKey
    );
    const liquidityProviderBTokenAcount = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      tokenBMintKeypair.publicKey,
      liquidityProviderKeypair.publicKey
    );
    await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      lpTokenMintKeypair.publicKey,
      liquidityProviderKeypair.publicKey
    );

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      tokenAMintKeypair.publicKey,
      liquidityProviderATokenAcount,
      provider.wallet.payer,
      BigInt(100_000)
    );
    await mintTo(
      provider.connection,
      provider.wallet.payer,
      tokenBMintKeypair.publicKey,
      liquidityProviderBTokenAcount,
      provider.wallet.payer,
      BigInt(200_000)
    );

    const balanceOfA = await provider.connection.getTokenAccountBalance(
      liquidityProviderATokenAcount
    );
    const balanceOfB = await provider.connection.getTokenAccountBalance(
      liquidityProviderBTokenAcount
    );

    expect(+balanceOfA.value.amount).to.equal(100_000);
    expect(+balanceOfB.value.amount).to.equal(200_000);
  });

  it("should add liquidity to exchange", async () => {
    const tx = await program.methods
      .addLiquidity(new BN(50_000), new BN(50_000))
      .accounts({
        poolState: poolStatePda,
        tokenAMint: tokenAMintKeypair.publicKey,
        tokenBMint: tokenBMintKeypair.publicKey,
        lpTokenMint: lpTokenMintKeypair.publicKey,
        poolTokenAVault: poolTokenAVaultKeypair.publicKey,
        poolTokenBVault: poolTokenBVaultKeypair.publicKey,
        payer: liquidityProviderKeypair.publicKey,
      })
      .signers([liquidityProviderKeypair])
      .rpc();

    console.log("Liquidity tx", tx);

    const accountA = await getAssociatedTokenAddress(
      tokenAMintKeypair.publicKey,
      liquidityProviderKeypair.publicKey
    );
    const accountB = await getAssociatedTokenAddress(
      tokenBMintKeypair.publicKey,
      liquidityProviderKeypair.publicKey
    );
    const accountLP = await getAssociatedTokenAddress(
      lpTokenMintKeypair.publicKey,
      liquidityProviderKeypair.publicKey
    );
    const balanceOfA = await provider.connection.getTokenAccountBalance(
      accountA
    );
    const balanceOfB = await provider.connection.getTokenAccountBalance(
      accountB
    );
    const balanceOfLP = await provider.connection.getTokenAccountBalance(
      accountLP
    );

    expect(+balanceOfA.value.amount).to.equal(50_000, "Balance A incorrect");
    expect(+balanceOfB.value.amount).to.equal(150_000, "Balance B incorrect");
    // sqrt (50_000 + 50_000) - 1_000 (minimum)
    expect(+balanceOfLP.value.amount).to.equal(49_000, "LP balance incorrect");
  });

  it("should user swap token A to token B", async () => {
    // create account
    await provider.connection.requestAirdrop(
      systemUser1Keypair.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 5
    );

    console.log("System user 1.", systemUser1Keypair.publicKey.toBase58());

    // create token accounts
    const systemUserATokenAcount = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      tokenAMintKeypair.publicKey,
      systemUser1Keypair.publicKey
    );
    const systemUserBTokenAcount = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      tokenBMintKeypair.publicKey,
      systemUser1Keypair.publicKey
    );

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      tokenAMintKeypair.publicKey,
      systemUserATokenAcount,
      provider.wallet.payer,
      BigInt(100_000)
    );

    const tx = await program.methods
      .swap(new BN(10_000), new BN(8_000))
      .accounts({
        poolState: poolStatePda,
        tokenFromMint: tokenAMintKeypair.publicKey,
        tokenToMint: tokenBMintKeypair.publicKey,
        poolTokenAVault: poolTokenAVaultKeypair.publicKey,
        poolTokenBVault: poolTokenBVaultKeypair.publicKey,
        payer: systemUser1Keypair.publicKey,
      })
      .signers([systemUser1Keypair])
      .rpc();

    console.log("Swap transaction.", tx);
    const accountA = await getAssociatedTokenAddress(
      tokenAMintKeypair.publicKey,
      systemUser1Keypair.publicKey
    );
    const accountB = await getAssociatedTokenAddress(
      tokenBMintKeypair.publicKey,
      systemUser1Keypair.publicKey
    );
    const balanceOfA = await provider.connection.getTokenAccountBalance(
      accountA
    );
    const balanceOfB = await provider.connection.getTokenAccountBalance(
      accountB
    );

    expect(+balanceOfA.value.amount).to.equal(90_000, "Balance A incorrect");
    expect(+balanceOfB.value.amount).to.equal(8_312, "Balance B incorrect");
  });
});
