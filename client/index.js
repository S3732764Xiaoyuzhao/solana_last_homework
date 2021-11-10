const web3 = require('@solana/web3.js');
const splToken = require('@solana/spl-token');
const BN = require('bn.js');

const decimals = 9;
const programId = new web3.PublicKey('5FtbYrH9uyXhvrMLUWxkDHjoK3JBzJEPE7X1JRvyExvE');
const seed = 'last_homework';
const token_programId = new web3.PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
const associated_token_programId = new web3.PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');
const sys = new web3.PublicKey('11111111111111111111111111111111');
const rent = new web3.PublicKey('SysvarRent111111111111111111111111111111111');

(async () => {
    // Connect to cluster
    var connection = new web3.Connection(
        'http://localhost:8899',
        'confirmed',
    );

    // Generate a new wallet keypair and airdrop SOL
    var fromWallet = web3.Keypair.generate();
    console.log('fromWallet: ->', fromWallet.publicKey.toBase58());
    var fromAirdropSignature = await connection.requestAirdrop(
        fromWallet.publicKey,
        web3.LAMPORTS_PER_SOL,
    );
    await connection.confirmTransaction(fromAirdropSignature);

    // Generate a new wallet to receive newly minted token
    var toWallet = web3.Keypair.generate();
    console.log('toWallet: ->', toWallet.publicKey.toBase58());
    var toAirdropSignature = await connection.requestAirdrop(
        toWallet.publicKey,
        web3.LAMPORTS_PER_SOL,
    );
    await connection.confirmTransaction(toAirdropSignature);

    //create new token mint
    let mint = await splToken.Token.createMint(
        connection,
        fromWallet,
        fromWallet.publicKey,
        null,
        decimals,
        splToken.TOKEN_PROGRAM_ID,
    );
    console.log('spl-token publicKey: ->', mint.publicKey.toBase58());

    //get the token account of the fromWallet Solana address, if it does not exist, create it
    let fromTokenAccount = await mint.getOrCreateAssociatedAccountInfo(
        fromWallet.publicKey,
    );
    console.log('fromTokenAccount: ->', fromTokenAccount.address.toBase58());

    //get the token account of the toWallet Solana address, if it does not exist, create it
    //user ada
    var toTokenAccount = await mint.getOrCreateAssociatedAccountInfo(
        toWallet.publicKey,
    );
    console.log('toTokenAccount: ->', toTokenAccount.address.toBase58());

    //program derived address
    var [programId_derived_pubkey, nonce] = await web3.PublicKey.findProgramAddress(
        [Buffer.from(seed)],
        programId
    );
    console.log('programId_derived_pubkey: ->', programId_derived_pubkey.toBase58());

    //get the token account of the program Solana address, if it does not exist, create it
    //Get program currency ada, not on the chain
    var programId_associated_address = await splToken.Token.getAssociatedTokenAddress(
        associated_token_programId,
        token_programId,
        mint.publicKey,
        programId_derived_pubkey,
        true
    );
    console.log('programId_associated_address: ->', programId_associated_address.toBase58());

    //get the token account of the user_derived Solana address, if it does not exist, create it
    //User derived address
    var user_derived_pubkey = await web3.PublicKey.createWithSeed(
        toWallet.publicKey,
        seed,
        programId
    );
    console.log('user_derived_pubkey: ->', user_derived_pubkey.toBase58());

    //Create an account for the user derived address
    const lamports = await connection.getMinimumBalanceForRentExemption(
        8,
    );
    const transaction_create_user_derived_account = new web3.Transaction().add(
        web3.SystemProgram.createAccountWithSeed({
            basePubkey: toWallet.publicKey,
            fromPubkey: toWallet.publicKey,
            lamports,
            newAccountPubkey: user_derived_pubkey,
            programId,
            seed,
            space: 8,
        })
    );
    var signature = await web3.sendAndConfirmTransaction(
        connection,
        transaction_create_user_derived_account,
        [toWallet],
        { commitment: 'confirmed' },
    );
    console.log('SIGNATURE', signature);
    console.log('transaction_create_user_derived_account success');

    //minting 100 new token to the "fromTokenAccount" account we just returned/created
    await mint.mintTo(
        fromTokenAccount.address,
        fromWallet.publicKey,
        [],
        100 * (10 ** decimals),
    );

    // Add token transfer instructions to transaction
    var transaction = new web3.Transaction().add(
        splToken.Token.createTransferInstruction(
            splToken.TOKEN_PROGRAM_ID,
            fromTokenAccount.address,
            toTokenAccount.address,
            fromWallet.publicKey,
            [],
            10 * (10 ** decimals),
        ),
    );

    // Sign transaction, broadcast, and confirm
    var signature = await web3.sendAndConfirmTransaction(
        connection,
        transaction,
        [fromWallet],
        { commitment: 'confirmed' },
    );
    console.log('SIGNATURE', signature);

    //Create the currency ada of the program derived address
    const instruction_create_program_ada = new web3.TransactionInstruction({
        keys: [
            { pubkey: toWallet.publicKey, isSigner: true, isWritable: true },
            { pubkey: programId_derived_pubkey, isSigner: false, isWritable: true },
            { pubkey: programId_associated_address, isSigner: false, isWritable: true },
            { pubkey: mint.publicKey, isSigner: false, isWritable: false },
            { pubkey: token_programId, isSigner: false, isWritable: false },
            { pubkey: associated_token_programId, isSigner: false, isWritable: true },
            { pubkey: sys, isSigner: false, isWritable: true },
            { pubkey: rent, isSigner: false, isWritable: true },
        ],
        programId,
        data: Buffer.from(Uint8Array.of(2)), // All instructions are hellos
    });
    var transaction = new web3.Transaction().add(
        instruction_create_program_ada
    );
    var signature = await web3.sendAndConfirmTransaction(
        connection,
        transaction,
        [toWallet],
        { commitment: 'confirmed' },
    );
    console.log('SIGNATURE', signature);

    //toWallet deposit 3 mint to program
    var num = 3 * (10 ** decimals);
    let indexData = [];
    for (const _value of new BN(num).toArray('le', 8)) {
        indexData.push(_value);
    }
    console.log(indexData);
    const instruction = new web3.TransactionInstruction({
        keys: [
            { pubkey: toWallet.publicKey, isSigner: true, isWritable: true },
            { pubkey: toTokenAccount.address, isSigner: false, isWritable: true },
            { pubkey: user_derived_pubkey, isSigner: false, isWritable: true },
            { pubkey: programId_associated_address, isSigner: false, isWritable: true },
            { pubkey: token_programId, isSigner: false, isWritable: false },
            { pubkey: mint.publicKey, isSigner: false, isWritable: false },],
        programId,
        data: Buffer.from(Uint8Array.of(0, ...indexData)), // All instructions are hellos
    });
    var deposit_transaction = new web3.Transaction().add(
        instruction
    );
    var signature = await web3.sendAndConfirmTransaction(
        connection,
        deposit_transaction,
        [toWallet],
        { commitment: 'confirmed' },
    );
    console.log('SIGNATURE', signature);

    //toWallet withdraw all deposits
    const instruction_claim = new web3.TransactionInstruction({
        keys: [{ pubkey: toWallet.publicKey, isSigner: true, isWritable: true },
        { pubkey: toTokenAccount.address, isSigner: false, isWritable: true },
        { pubkey: user_derived_pubkey, isSigner: false, isWritable: true },
        { pubkey: programId_derived_pubkey, isSigner: false, isWritable: trues },
        { pubkey: programId_associated_address, isSigner: false, isWritable: true },
        { pubkey: token_programId, isSigner: false, isWritable: false },
        { pubkey: mint.publicKey, isSigner: false, isWritable: false },],
        programId,
        data: Buffer.from(Uint8Array.of(1, nonce)), // All instructions are hellos
    });
    var deposit_transaction = new web3.Transaction().add(
        instruction_claim
    );
    var signature = await web3.sendAndConfirmTransaction(
        connection,
        deposit_transaction,
        [toWallet],
        { commitment: 'confirmed' },
    );
    console.log('SIGNATURE', signature);

})();
