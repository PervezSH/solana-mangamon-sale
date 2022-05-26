import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PublicKey } from '@solana/web3.js';
import { SolanaMangamonSale } from "../target/types/solana_mangamon_sale";
import { expect } from 'chai';

// Configure the client to use the local cluster.
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.SolanaMangamonSale as Program<SolanaMangamonSale>;

// Create an account keypair for our program to use.
const authorizedSaleAccount = anchor.web3.Keypair.generate();
const saleAccount = anchor.web3.Keypair.generate();

async function initializateAccount() {
    await program.methods
        .initialize(
            new anchor.BN(4000),
            new anchor.BN(1652972400),
            new anchor.BN(1653285600),
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

    await program.methods
        .setInitialPercentageAllocationIdoTokens(
            20
        )
        .accounts({
            authorizedSaleAccount: authorizedSaleAccount.publicKey,
            saleAccount: saleAccount.publicKey,
            admin: provider.wallet.publicKey
        })
        .rpc();
}

describe("solana-mangamon-sale", () => {
    before(async function () {
        try {
            await initializateAccount();
        } catch (error) {
            console.log(error);
        }
    });
    describe("#initialization", function () {
        it("Checks if everything initialized correctly!", async function () {
            expect((await program.account.authorizedSaleAccount
                .fetch(authorizedSaleAccount.publicKey)).idoTokenPriceRatio.toNumber()).to.equal(4000);
            expect((await program.account.authorizedSaleAccount
                .fetch(authorizedSaleAccount.publicKey)).idoTokenPriceMultiplier.toNumber()).to.equal(10000);
            expect((await program.account.authorizedSaleAccount
                .fetch(authorizedSaleAccount.publicKey)).initialPercentageAllocationIdoTokens).to.equal(20);
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
                .fetch(authorizedSaleAccount.publicKey)).endDateFunding.toNumber()).to.equal(1653285600);
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
    });
});