import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PublicKey } from '@solana/web3.js';
import { SolanaMangamonSale } from "../target/types/solana_mangamon_sale";
import { expect } from 'chai';

describe("solana-mangamon-sale", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaMangamonSale as Program<SolanaMangamonSale>;

  // Create an account keypair for our program to use.
  const authorizedSaleAccount = anchor.web3.Keypair.generate();
  const saleAccount = anchor.web3.Keypair.generate();

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize(
        new anchor.BN(4000),
        new anchor.BN(1652972400),
        new anchor.BN(1654285600),
        new anchor.BN(1666504800),
        20,
        false,
      )
      .accounts({
        authorizedSaleAccount: authorizedSaleAccount.publicKey,
        saleAccount: saleAccount.publicKey,
        user: provider.wallet.publicKey,
      })
      .signers([authorizedSaleAccount, saleAccount])
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Should check initialized fields!", async function () {
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).idoTokenPriceRatio.toNumber()).to.equal(4000);
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).idoTokenPriceMultiplier.toNumber()).to.equal(10000);
    // expect((await program.account.authorizedSaleAccount
    //   .fetch(authorizedSaleAccount.publicKey)).initialPercentageAllocationIdoTokens).to.equal(20);
    expect((await program.account.saleAccount
      .fetch(saleAccount.publicKey)).totalSpendPayTokens.toNumber()).to.equal(0);
    expect((await program.account.saleAccount
      .fetch(saleAccount.publicKey)).totalAllocatedIdoTokens.toNumber()).to.equal(0);
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).tokensForSale.toNumber()).to.equal(0);
    expect((await program.account.saleAccount
      .fetch(saleAccount.publicKey)).investorCount.toNumber()).to.equal(0);
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).startDateFunding.toNumber()).to.equal(1652972400);
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).endDateFunding.toNumber()).to.equal(1654285600);
    // expect((await program.account.authorizedSaleAccount
    //   .fetch(authorizedSaleAccount.publicKey)).startDateOfClaimingTokens.toNumber()).to.equal(0);
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).endDateOfClaimingTokens.toNumber()).to.equal(1666504800);
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).isIdoTokenFundedToContract).to.equal(false);
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).isFundingCanceled).to.equal(false);
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).inOneTransaction).to.equal(false);
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).isClaimingOpen).to.equal(false);
  });

  it("Should check initialPercentageAllocationIdoTokens field!", async function () {
    await program.methods
      .setInitialPercentageAllocationIdoTokens(20)
      .accounts({
        authorizedSaleAccount: authorizedSaleAccount.publicKey,
        admin: provider.wallet.publicKey,
      })
      .rpc();
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).initialPercentageAllocationIdoTokens).to.equal(20);
  });

  it("Should check buyer's info", async function () {
    const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("buyer-info"),
        provider.wallet.publicKey.toBuffer()
      ],
      program.programId
    );
    await program.methods
      .creatBuyerInfo(
        new anchor.BN(109430000),
        new anchor.BN(2735700000000000),
      )
      .accounts({
        user: provider.wallet.publicKey,
        buyerInfo: buyerInfoPDA,
      })
      .rpc();
    expect((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).spendPayTokens.toNumber()).to.equal(109430000);
    expect((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).idoTokensToGet.toNumber()).to.equal(2735700000000000);
    expect((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).idoTokensClaimed.toNumber()).to.equal(0);
    expect((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).hasClaimedPayTokens).to.equal(false);
  });

  it("Should check start date of claiming tokens", async function () {
    try {
      await program.methods
        .enableClaiming(
          true,
          new anchor.BN(1653385754)
        )
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          admin: provider.wallet.publicKey,
        })
        .rpc();
    } catch (error) {
      console.log(error);
    }
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).isClaimingOpen).to.equal(true);
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).startDateOfClaimingTokens.toNumber()).to.equal(1653385754);
  });

  it("Should check end date of claiming tokens", async function () {
    let e: any;
    try {
      await program.methods
        .setEndDateOfClaimingTokens(
          new anchor.BN(1663385754)
        )
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          admin: provider.wallet.publicKey,
        })
        .rpc();
    } catch (error) {
      e = error;
    }
    const stringifiedError = JSON.stringify(e);
    expect(stringifiedError.includes("Claiming is already enabled")).to.equal(true);
  });

  it("Should throw error while adding fund to the program address", async function () {
    let e: any;
    try {
      await program.methods
        .fundToContract(
          new anchor.BN(14735370000)
        )
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          admin: provider.wallet.publicKey,
        })
        .rpc();
    } catch (error) {
      e = error;
    }
    const stringifiedError = JSON.stringify(e);
    expect(stringifiedError.includes("The Funding Period has not ended")).to.equal(true);
  });

  it("Should throw error while withdrawing pay tokens", async function () {
    let e: any;
    try {
      await program.methods
        .withdrawPayTokens(
          new anchor.BN(10000000)
        )
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          admin: provider.wallet.publicKey,
        })
        .rpc();
    } catch (error) {
      e = error;
    }
    const stringifiedError = JSON.stringify(e);
    expect(stringifiedError.includes("The Funding Period has not ended")).to.equal(true);
  });

  it("Should throw error while withdrawing unsold ido tokens", async function () {
    let e: any;
    try {
      await program.methods
        .withdrawUnsoldIdoTokens(
          new anchor.BN(10000000)
        )
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          admin: provider.wallet.publicKey,
        })
        .rpc();
    } catch (error) {
      e = error;
    }
    const stringifiedError = JSON.stringify(e);
    expect(stringifiedError.includes("The Funding Period has not ended")).to.equal(true);
  });

  it("Should cancle the entire sale", async function () {
    try {
      await program.methods
        .cancelIdoSale()
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          admin: provider.wallet.publicKey,
        })
        .rpc();
    } catch (error) {
      console.log(error);
    }
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).isFundingCanceled).to.equal(true);
  });

  it("Should calculates how much Payment tokens needed to acquire IDO token allocation", async function () {
    try {
      const returnData = await program.methods
        .calculateMaxPaymentToken(
          new anchor.BN("10000000000000000")
        )
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          user: provider.wallet.publicKey,
        })
        .view();
      expect(returnData.toNumber()).to.equal(4000);
    } catch (error) {
      console.log(error);
    }
  });
});