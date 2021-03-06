import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PublicKey } from '@solana/web3.js';
import { SolanaMangamonSale } from "../target/types/solana_mangamon_sale";
import { expect } from 'chai';

// Configure the client to use the local cluster.
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.SolanaMangamonSale as Program<SolanaMangamonSale>;

async function initializateAccount(authorizedSaleAccount: anchor.web3.Keypair, saleAccount: anchor.web3.Keypair) {
    await program.methods
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
    describe("#initialization", function () {
        // Create an account keypair for our program to use.
        const authorizedSaleAccount = anchor.web3.Keypair.generate();
        const saleAccount = anchor.web3.Keypair.generate();
        before(async function () {
            try {
                await initializateAccount(authorizedSaleAccount, saleAccount);
            } catch (error) {
                console.log(error);
            }
        });
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
                .fetch(authorizedSaleAccount.publicKey)).endDateFunding.toNumber()).to.equal(1654285600);
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
            // Create an account keypair for our program to use.
            const authorizedSaleAccount = anchor.web3.Keypair.generate();
            const saleAccount = anchor.web3.Keypair.generate();
            before(async function () {
                try {
                    await initializateAccount(authorizedSaleAccount, saleAccount);
                } catch (error) {
                    console.log(error);
                }
            });
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
            // Create an account keypair for our program to use.
            const authorizedSaleAccount = anchor.web3.Keypair.generate();
            const saleAccount = anchor.web3.Keypair.generate();
            before(async function () {
                try {
                    await initializateAccount(authorizedSaleAccount, saleAccount);
                } catch (error) {
                    console.log(error);
                }
            });
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
        describe("#setEndDateOfClaimingTokens()", function () {
            // Create an account keypair for our program to use.
            const authorizedSaleAccount = anchor.web3.Keypair.generate();
            const saleAccount = anchor.web3.Keypair.generate();
            before(async function () {
                try {
                    await initializateAccount(authorizedSaleAccount, saleAccount);
                } catch (error) {
                    console.log(error);
                }
            });
            it("Should change the end date for tokens to be claimed!", async function () {
                try {
                    await program.methods
                        .setEndDateOfClaimingTokens(
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
                    .fetch(authorizedSaleAccount.publicKey)).endDateOfClaimingTokens.toNumber()).to.equal(1656090000);
            });
            it("Should throw error, as claiming is already enabled!", async function () {
                let e: any;
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
                    await program.methods
                        .setEndDateOfClaimingTokens(
                            new anchor.BN(1656090000)
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
                expect(JSON.stringify(e).includes("Claiming is already enabled")).to.equal(true);
            });
        });
    });
    describe("#business logic", function () {
        describe("#calculateMaxPaymentToken()", function () {
            // Create an account keypair for our program to use.
            const authorizedSaleAccount = anchor.web3.Keypair.generate();
            const saleAccount = anchor.web3.Keypair.generate();
            before(async function () {
                try {
                    await initializateAccount(authorizedSaleAccount, saleAccount);
                } catch (error) {
                    console.log(error);
                }
            });
            it("Should calculate the payment token need to acquire the IDO token allocation!", async function () {
                let returnData: anchor.BN;
                try {
                    returnData = await program.methods
                        .calculateMaxPaymentToken(
                            new anchor.BN("22345623767423223324")
                        )
                        .accounts({
                            authorizedSaleAccount: authorizedSaleAccount.publicKey,
                            saleAccount: saleAccount.publicKey,
                            user: provider.wallet.publicKey,
                        })
                        .view();
                } catch (error) {
                    console.log(error)
                }
                expect(returnData.toNumber()).to.equal(8936000);
            });
        });
        describe("#calculateIdoTokensBought()", function () {
            // Create an account keypair for our program to use.
            const authorizedSaleAccount = anchor.web3.Keypair.generate();
            const saleAccount = anchor.web3.Keypair.generate();
            before(async function () {
                try {
                    await initializateAccount(authorizedSaleAccount, saleAccount);
                } catch (error) {
                    console.log(error);
                }
            });
            it("Should calculate the amount of IDO token bought!", async function () {
                let returnData: anchor.BN;
                try {
                    returnData = await program.methods
                        .calculateIdoTokensBought(
                            new anchor.BN("8936000")
                        )
                        .accounts({
                            authorizedSaleAccount: authorizedSaleAccount.publicKey,
                            saleAccount: saleAccount.publicKey,
                            user: provider.wallet.publicKey,
                        })
                        .view();
                } catch (error) {
                    console.log(error)
                }
                expect(String(returnData)).to.equal("22340000000000000000");
            });
        });
        describe("#fundToContract()", function () {
            // Create an account keypair for our program to use.
            const authorizedSaleAccount = anchor.web3.Keypair.generate();
            const saleAccount = anchor.web3.Keypair.generate();
            before(async function () {
                try {
                    await program.methods
                        .initialize(
                            new anchor.BN(4000),
                            new anchor.BN(1652972400),
                            new anchor.BN(1653646545),
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
                } catch (error) {
                    console.log(error);
                }
            });
            it("Should calculate the amount of IDO token bought!", async function () {
                try {
                    await program.methods
                        .fundToContract(
                            new anchor.BN("14735370000000000000000")
                        )
                        .accounts({
                            authorizedSaleAccount: authorizedSaleAccount.publicKey,
                            saleAccount: saleAccount.publicKey,
                            admin: provider.wallet.publicKey,
                        })
                        .rpc();
                } catch (error) {
                    console.log(error)
                }
                expect(String((await program.account.authorizedSaleAccount
                    .fetch(authorizedSaleAccount.publicKey)).tokensForSale)).to.equal("14735370000000000000000");
                expect((await program.account.authorizedSaleAccount
                    .fetch(authorizedSaleAccount.publicKey)).isIdoTokenFundedToContract).to.equal(true);
            });
            it(`Should throw error, saying "Already funded tokens"!`, async function () {
                let e: any;
                try {
                    await program.methods
                        .fundToContract(
                            new anchor.BN("14735370000000000000000")
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
                expect(JSON.stringify(e).includes("Already funded tokens")).to.equal(true);
            });
        });
        describe("#buy()", function () {
            // Create an account keypair for our program to use.
            const authorizedSaleAccount = anchor.web3.Keypair.generate();
            const saleAccount = anchor.web3.Keypair.generate();
            let e: any;
            before(async function () {
                try {
                    await initializateAccount(authorizedSaleAccount, saleAccount);
                } catch (error) {
                    console.log(error);
                }
            });
            it("Should let user buy IDO tokens worth of 4000 pay tokens!", async function () {
                const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
                    [
                        anchor.utils.bytes.utf8.encode("buyer-info"),
                        provider.wallet.publicKey.toBuffer()
                    ],
                    program.programId
                );
                try {
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
            it(`Should throw error saying, "You cannot buy more tokens than is allowed according to your lottery allocation calculation"!`, async function () {
                const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
                    [
                        anchor.utils.bytes.utf8.encode("buyer-info"),
                        provider.wallet.publicKey.toBuffer()
                    ],
                    program.programId
                );
                try {
                    await program.methods
                        .buy(
                            new anchor.BN(2000)
                        )
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
                expect(JSON.stringify(e).includes("You cannot buy more tokens than is allowed according to your lottery allocation calculation")).to.equal(true);
            });
        });
        describe("#claimTokens()", function () {
            // Create an account keypair for our program to use.
            const authorizedSaleAccount = anchor.web3.Keypair.generate();
            const saleAccount = anchor.web3.Keypair.generate();
            let e: any;
            before(async function () {
                try {
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
                } catch (error) {
                    console.log(error);
                }
            });
            it(`Should throw error saying "Tokens have not been added to the contract YET"!`, async function () {
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
                const stringifiedError = JSON.stringify(e);
                expect(stringifiedError.includes("Tokens have not been added to the contract YET")).to.equal(true);
            });
            it(`Should throw error saying "Cannot claim, you need to wait until claiming is enabled"!`, async function () {
                const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
                    [
                        anchor.utils.bytes.utf8.encode("buyer-info"),
                        provider.wallet.publicKey.toBuffer()
                    ],
                    program.programId
                );
                try {
                    await program.methods
                        .fundToContract(
                            new anchor.BN("14735370000000000000000")
                        )
                        .accounts({
                            authorizedSaleAccount: authorizedSaleAccount.publicKey,
                            saleAccount: saleAccount.publicKey,
                            admin: provider.wallet.publicKey,
                        })
                        .rpc();
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
                const stringifiedError = JSON.stringify(e);
                expect(stringifiedError.includes("Cannot claim, you need to wait until claiming is enabled")).to.equal(true);
            });
            it(`Should throw error saying "You are not a buyer"!`, async function () {
                const [buyerInfoPDA, _] = await PublicKey.findProgramAddress(
                    [
                        anchor.utils.bytes.utf8.encode("buyer-info"),
                        provider.wallet.publicKey.toBuffer()
                    ],
                    program.programId
                );
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
                const stringifiedError = JSON.stringify(e);
                expect(stringifiedError.includes("You are not a buyer")).to.equal(true);
            });
            it("Should let buyer claim all IDO tokens he has bought!", async function () {
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
                expect(String((await program.account.buyerInfo
                    .fetch(buyerInfoPDA)).idoTokensClaimed)).to.equal("0");
            });
        });
    });
});