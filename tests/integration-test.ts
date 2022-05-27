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

async function createPDA(_buyer: PublicKey) {
    const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
        [
            anchor.utils.bytes.utf8.encode("buyer-info"),
            _buyer.toBuffer()
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
    return buyerInfoPDA
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
        it("Should create PDA for a buyer and initialized its field!", async function () {
            let buyerInfoPDA: PublicKey;
            try {
                buyerInfoPDA = await createPDA(provider.wallet.publicKey);
            } catch (error) {
                console.log(error)
            }
            expect((await program.account.buyerInfo
                .fetch(buyerInfoPDA)).spendPayTokens.toNumber()).to.equal(0);
            expect((await program.account.buyerInfo
                .fetch(buyerInfoPDA)).idoTokensToGet.toNumber()).to.equal(0);
            expect((await program.account.buyerInfo
                .fetch(buyerInfoPDA)).idoTokensClaimed.toNumber()).to.equal(0);
            expect((await program.account.buyerInfo
                .fetch(buyerInfoPDA)).hasClaimedPayTokens).to.equal(false);
        });
    });
    describe("#setters", function () {
        describe("#setInitialPercentageAllocationIdoTokens()", function () {
            let e: any;
            it("Should change initial percentage of token allocation to 25!", async function () {
                try {
                    await program.methods
                        .setInitialPercentageAllocationIdoTokens(
                            25
                        )
                        .accounts({
                            authorizedSaleAccount: authorizedSaleAccount.publicKey,
                            saleAccount: saleAccount.publicKey,
                            admin: provider.wallet.publicKey
                        })
                        .rpc();
                } catch (error) {
                    e = error;
                }
                expect((await program.account.authorizedSaleAccount
                    .fetch(authorizedSaleAccount.publicKey)).initialPercentageAllocationIdoTokens).to.equal(25);
            });
            it("Should throw error, as percentage can't be greater than 100!", async function () {
                try {
                    await program.methods
                        .setInitialPercentageAllocationIdoTokens(
                            120
                        )
                        .accounts({
                            authorizedSaleAccount: authorizedSaleAccount.publicKey,
                            saleAccount: saleAccount.publicKey,
                            admin: provider.wallet.publicKey
                        })
                        .rpc();
                } catch (error) {
                    e = error;
                }
                expect(JSON.stringify(e).includes("You cannot give more than 100 percent of the token allocation")).to.equal(true);
            });
        });
        describe("#enableClaiming()", function () {
            it("Should enable isClaimingOpen, and initialize the field startDateOfClaimingTokens!", async function () {
                try {
                    await program.methods
                        .enableClaiming(
                            true,
                            new anchor.BN(1656090000)
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
                    .fetch(authorizedSaleAccount.publicKey)).startDateOfClaimingTokens.toNumber()).to.equal(1656090000);
            });
        });
    });
});