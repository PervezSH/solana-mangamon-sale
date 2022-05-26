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

  // Account initialization
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
        new anchor.BN(0),
        new anchor.BN(0),
      )
      .accounts({
        user: provider.wallet.publicKey,
        buyerInfo: buyerInfoPDA,
      })
      .rpc();
    expect((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).spendPayTokens.toNumber()).to.equal(0);
    expect((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).idoTokensToGet.toNumber()).to.equal(0);
    expect((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).idoTokensClaimed.toNumber()).to.equal(0);
    expect((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).hasClaimedPayTokens).to.equal(false);
  });

  // Setters
  it("Should check initialPercentageAllocationIdoTokens field!", async function () {
    await program.methods
      .setInitialPercentageAllocationIdoTokens(20)
      .accounts({
        authorizedSaleAccount: authorizedSaleAccount.publicKey,
        saleAccount: saleAccount.publicKey,
        admin: provider.wallet.publicKey,
      })
      .rpc();
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).initialPercentageAllocationIdoTokens).to.equal(20);
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
          saleAccount: saleAccount.publicKey,
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
          saleAccount: saleAccount.publicKey,
          admin: provider.wallet.publicKey,
        })
        .rpc();
    } catch (error) {
      e = error;
    }
    const stringifiedError = JSON.stringify(e);
    expect(stringifiedError.includes("Claiming is already enabled")).to.equal(true);
  });

  // Getters
  it("Should return list of all buyers", async function () {
    let returnData: any;
    try {
      returnData = await program.methods
        .getBuyers()
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          user: provider.wallet.publicKey
        })
        .view();
    } catch (error) {
      console.log(error);
    }
    expect(returnData.length).to.equal(0);
  });

  it("Should check if the provided wallet address is buyer", async function () {
    let returnData: boolean;
    try {
      const user = anchor.web3.Keypair.generate();
      returnData = await program.methods
        .isBuyer(
          user.publicKey
        )
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          user: provider.wallet.publicKey
        })
        .view();
    } catch (error) {
      console.log(error);
    }
    expect(returnData).to.equal(false);
  });

  it("Should retrieve total tokens bought and spent by a user", async function () {
    const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("buyer-info"),
        provider.wallet.publicKey.toBuffer()
      ],
      program.programId
    );
    let returnData: any;
    try {
      returnData = await program.methods
        .getTotalIdoTokensBoughtAndPayTokensSpend(
          provider.wallet.publicKey
        )
        .accounts({
          user: provider.wallet.publicKey,
          buyerInfo: buyerInfoPDA,
          saleAccount: saleAccount.publicKey,
          authorizedSaleAccount: authorizedSaleAccount.publicKey
        })
        .view();
    } catch (error) {
      console.log(error);
    }
    expect(String(returnData)).to.equal("0,0");
  });

  it("Should retrieve total tokens bought and spent by all the user", async function () {
    let returnData: any;
    try {
      returnData = await program.methods
        .totalIdoTokensBoughtAndPayTokensSpend()
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          user: provider.wallet.publicKey
        })
        .view();
    } catch (error) {
      console.log(error);
    }
    expect(String(returnData)).to.equal("0,0");
  });

  it("Should return the claimable tokens at this point in time for a user", async function () {
    const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("buyer-info"),
        provider.wallet.publicKey.toBuffer()
      ],
      program.programId
    );
    let returnData: any;
    try {
      returnData = await program.methods
        .getClaimableTokens(
          provider.wallet.publicKey
        )
        .accounts({
          user: provider.wallet.publicKey,
          buyerInfo: buyerInfoPDA,
          saleAccount: saleAccount.publicKey,
          authorizedSaleAccount: authorizedSaleAccount.publicKey
        })
        .view();
    } catch (error) {
      console.log(error);
    }
    expect(String(returnData)).to.equal("0");
  });

  // Business Logic
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

  it("Should calculate the amount of Ido Tokens bought", async function () {
    let returnData: any;
    try {
      returnData = await program.methods
        .calculateIdoTokensBought(
          new anchor.BN("250000000")
        )
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          user: provider.wallet.publicKey,
        })
        .view();
    } catch (error) {
      console.log(error);
    }
    expect(String(returnData)).to.equal("625000000000000000000");
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

  it("Should check if user can buy IDO tokens", async function () {
    const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("buyer-info"),
        provider.wallet.publicKey.toBuffer()
      ],
      program.programId
    );
    try {
      // cancel funding
      await program.methods
        .cancelIdoSale()
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          admin: provider.wallet.publicKey,
        })
        .rpc();
      await program.methods
        .buy(
          new anchor.BN(4000)
        )
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          buyerInfo: buyerInfoPDA,
          user: provider.wallet.publicKey
        })
        .rpc();
    } catch (error) {
      console.log(error);
    }
    expect((await program.account.saleAccount
      .fetch(saleAccount.publicKey)).investorCount.toNumber()).to.equal(1);
    expect(String((await program.account.saleAccount
      .fetch(saleAccount.publicKey)).totalSpendPayTokens)).to.equal("4000");
    expect(String((await program.account.saleAccount
      .fetch(saleAccount.publicKey)).totalAllocatedIdoTokens)).to.equal("10000000000000000");
    expect(String((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).spendPayTokens)).to.equal("4000");
    expect(String((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).idoTokensToGet)).to.equal("10000000000000000");
  });

  it("should check if user can claim their IDO tokens after funding period", async function () {
    let e: any;
    const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("buyer-info"),
        provider.wallet.publicKey.toBuffer()
      ],
      program.programId
    );
    try {
      await program.methods
        .claimTokens()
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          buyerInfo: buyerInfoPDA,
          user: provider.wallet.publicKey
        })
        .rpc();
    } catch (error) {
      e = error;
    }
    // expect(String((await program.account.buyerInfo
    //   .fetch(buyerInfoPDA)).idoTokensClaimed)).to.equal("4000");
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
          saleAccount: saleAccount.publicKey,
          admin: provider.wallet.publicKey,
        })
        .rpc();
    } catch (error) {
      console.log(error);
    }
    expect((await program.account.authorizedSaleAccount
      .fetch(authorizedSaleAccount.publicKey)).isFundingCanceled).to.equal(true);
  });

  it("Should check if user can claim payed token if funding is canceled", async function () {
    const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("buyer-info"),
        provider.wallet.publicKey.toBuffer()
      ],
      program.programId
    );
    try {
      await program.methods
        .claimPayedTokensOnIdoCancel()
        .accounts({
          authorizedSaleAccount: authorizedSaleAccount.publicKey,
          saleAccount: saleAccount.publicKey,
          buyerInfo: buyerInfoPDA,
          user: provider.wallet.publicKey
        })
        .rpc();
    } catch (error) {
      console.log(error);
    }
    expect((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).spendPayTokens.toNumber()).to.equal(0);
    expect((await program.account.buyerInfo
      .fetch(buyerInfoPDA)).hasClaimedPayTokens).to.equal(true);
  });
});