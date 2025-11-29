import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorAmm } from "../target/types/anchor_amm";
import { Keypair, PublicKey } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createMint, getAssociatedTokenAddress, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { expect } from "chai";
import { seed } from "@coral-xyz/anchor/dist/cjs/idl";
import { BN } from "bn.js";

describe("anchor-amm", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let connection = provider.connection;
  
  const program = anchor.workspace.anchorAmm as Program<AnchorAmm>;
  const payer = provider.wallet ;


  //SEED
  //FEE
  //AUTHORITY(SIGNER) and airdrop to him sol..
  //MINTX 
  //MINTY
  //MINT_LP(PDA)
  //CONFIG(PDA)
  //VAULT_X
  //VAULT_Y
  // USER ATA X
  //USER ATA Y
  //USER ATA LP
  //TOKEN_PROGRAM
  //ASSOCIATED_TOKEN_PROGRAM
  //SYSTEM_PROGRAM

  let seed_config: anchor.BN;
  let seed_lp: anchor.BN;
  let authority: Keypair;
  let user: Keypair;

  let mint_x: PublicKey;
  let mint_y: PublicKey;
  let mint_lp: PublicKey;
  let config_PDA: PublicKey;
  
  let user_ata_x: PublicKey;
  let user_ata_y: PublicKey;
  let user_ata_lp: PublicKey;
  let vault_x_ata: PublicKey;
  let vault_y_ata: PublicKey;


  let config_bump: number;
  let lp_bump: number;

  
before("Setting up accounts: ",async() => {

    seed_config = new anchor.BN(2);
    seed_lp  = new anchor.BN(4);
    authority = Keypair.generate();
    user = Keypair.generate();

    const authority_airdrop_signature = await connection.requestAirdrop(
      authority.publicKey,
      100*anchor.web3.LAMPORTS_PER_SOL,
    );

    const tx_signature = await connection.confirmTransaction({
      signature: authority_airdrop_signature,
      blockhash: (await connection.getLatestBlockhash()).blockhash,
      lastValidBlockHeight: (await connection.getLatestBlockhash()).lastValidBlockHeight
    });

    console.log("Airdrop to authority confirmed: ", authority_airdrop_signature);

    const user_airdrop_signature = await connection.requestAirdrop(
      user.publicKey,
      100*anchor.web3.LAMPORTS_PER_SOL,
    );

    const user_airdrop_tx_signature = await connection.confirmTransaction({
      signature: user_airdrop_signature,
      blockhash: (await connection.getLatestBlockhash()).blockhash,
      lastValidBlockHeight: (await connection.getLatestBlockhash()).lastValidBlockHeight
    });

    console.log("Airdrop to user confirmed: ", user_airdrop_signature);


    //CREATING MINTS..
    mint_x = await createMint(
      connection,
      payer.payer,
      authority.publicKey, //e mint Authority
      null,
      6
    );

    console.log("Mint X Created at : ", mint_x.toBase58());
    
    mint_y = await createMint(
      connection,
      payer.payer,
      authority.publicKey, //e token owner wass off curv
      null,
      6
    );

    console.log("Mint Y Created at : ", mint_y.toBase58());

    [config_PDA,config_bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("config"),
        seed_config.toArrayLike(Buffer,"le",8)
      ],
      program.programId
    );

    [mint_lp,lp_bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("lp"),
        config_PDA.toBuffer(),
      ],
      program.programId
    );

    console.log("Config PDA: ",config_PDA.toBase58());
    console.log("Config Bump: ",config_bump);

    console.log("Mint PDA: ",mint_lp.toBase58());
    console.log("Mint bump: ",lp_bump);


    //Creating ATA's
    user_ata_x = (await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      mint_x,
      user.publicKey,
    )).address;

    console.log("User ATA X Created ....",user_ata_x.toBase58());


    user_ata_y = (await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      mint_y,
      user.publicKey
    )).address;

     console.log("User ATA Y Created ....",user_ata_y.toBase58());

    //  user_ata_lp = (await getOrCreateAssociatedTokenAccount(
    //   connection,
    //   user,
    //   mint_lp,
    //   user.publicKey,
    //   true
    //  )).address;


    const mint_transaction_x = await mintTo(
      connection,
      authority,
      mint_x,
      user_ata_x,
      authority.publicKey,
      1000*10**6
    );

    console.log("1000 X Tokens minted to user. Signature: ",mint_transaction_x);


    const mint_transaction_y = await mintTo(
      connection,
      authority,
      mint_y,
      user_ata_y,
      authority.publicKey,
      1000*10**6
    );

    console.log("1000 Y Tokens minted to user. Signature: ",mint_transaction_y);

    user_ata_lp = await getAssociatedTokenAddress(
      mint_lp,
      user.publicKey,
    );

     console.log("User LP ATA Created...:" ,user_ata_lp.toBase58());

     vault_x_ata = await getAssociatedTokenAddress(
      mint_x,
      config_PDA,
      true //e allow the owner to be a pda
     );


     console.log("Vault ATA X created...",vault_x_ata.toBase58());

     
     vault_y_ata = await getAssociatedTokenAddress(
      mint_y,
      config_PDA,
      true, //allow the owner to be a pda
     );


     console.log("Vault ATA Y created...",vault_y_ata.toBase58());
  })
  

  //WORKING...
  it("Is initialized!", async () => {
    // Add your test here.
    const fee = 100;
    const tx = await program.methods.initialize(seed_config,100,authority.publicKey).accountsPartial({
      initializer: authority.publicKey,
      mintX:  mint_x,
      mintY: mint_y,
      mintLp: mint_lp,
      config: config_PDA,
      vaultX: vault_x_ata,
      vaultY: vault_y_ata,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SYSTEM_PROGRAM_ID,
    }).signers([authority]).rpc();
    console.log("Your Initialize transaction signature", tx);
    const configState = await program.account.config.fetch(config_PDA);
    expect(configState.authority.toBase58()).to.equal(authority.publicKey.toBase58());
  });


  //WORKING...
  it("Deposit", async() => {
    const amount = new BN(4*10**6);
    let max_x = new BN(5*10**6);
    let max_y = new BN(5*10**6);
    const tx = await program.methods.deposit(seed_config,amount,max_x,max_y).accountsPartial({
      user: user.publicKey,
      mintX: mint_x,
      mintY: mint_y,
      mintLp: mint_lp,
      config: config_PDA,
      userX: user_ata_x,
      userY: user_ata_y,
      vaultX: vault_x_ata,
      vaultY: vault_y_ata,
      userLp: user_ata_lp
    }).signers([user]).rpc();
    console.log("Deposit working. Transaction signature: ", tx);
    const vaultXState = await connection.getTokenAccountBalance(vault_x_ata);
    console.log("Vault X Balance after deposit: ",vaultXState.value.amount);

    const vaultYState = await connection.getTokenAccountBalance(vault_y_ata);
    console.log("Vault Y Balance after deposit: ",vaultYState.value.amount);

    const userLPTokenBalanceafterDeposit = await connection.getTokenAccountBalance(user_ata_lp);
    console.log(
      "User LP Balance after deposit: ", userLPTokenBalanceafterDeposit.value.amount
    )
  })

  //WORKING...
  it("Withdraw",async() => {
    const amount = new BN(3*10**6);
    let min_x = new BN(2.5*10**6)
    let min_y = new BN(2.5*10**6)
    const tx = await program.methods.withdraw(seed_config,amount,min_x,min_y).accountsPartial({
      user: user.publicKey,
      mintX: mint_x,
      mintY: mint_y,
      mintLp: mint_lp,
      config: config_PDA,
      userX: user_ata_x,
      userY: user_ata_y,
      vaultX: vault_x_ata,
      vaultY: vault_y_ata,
      userLp: user_ata_lp
    }).signers([user]).rpc();
   console.log("Withdraw working. Transaction signature: ", tx);
   const vaultXState = await connection.getTokenAccountBalance(vault_x_ata);
   console.log("Vault_X Balance after withdraw: ",vaultXState.value.amount);

    const vaultYState = await connection.getTokenAccountBalance(vault_y_ata);
   console.log("Vault_Y Balance after withdraw: ",vaultYState.value.amount);

   const userLPTokenBalanceafterDeposit = await connection.getTokenAccountBalance(user_ata_lp);
   console.log(
      "User LP Balance after withdraw: ", userLPTokenBalanceafterDeposit.value.amount
   )
  })

  //WORKING...
  it("Swap",async() => {

    const amount = new BN(4*10**6);
    const min = new BN(2*10**6)
    const is_x =  false

    const userXBalanceBeforeDeposit = await connection.getTokenAccountBalance(user_ata_x);
    console.log("User X Balance Before Second Deposit", userXBalanceBeforeDeposit.value.amount);

    const userYBalanceBeforeDeposit = await connection.getTokenAccountBalance(user_ata_y);
    console.log("User Y Balance Before Second Deposit", userYBalanceBeforeDeposit.value.amount);

    
    const vault_x_ata_balance_before_deposit = await connection.getTokenAccountBalance(vault_x_ata);
    console.log("Vault_X ATA Balance Before Deposit: ",vault_x_ata_balance_before_deposit.value.amount);

    const vault_y_ata_balance_before_deposit = await connection.getTokenAccountBalance(vault_y_ata);
    console.log("Vault_Y ATA Balance Before Deposit : ",vault_y_ata_balance_before_deposit.value.amount);

    // const configState = await program.account.config.fetch(config_PDA);
    // expect(configState.authority.toBase58()).to.equal(authority.publicKey.toBase58());
    // console.log("State of the Pool before second deposit : ..",configState);

    console.log("Depositing again before the swap: ....");
    const amount_deposit = new BN(4*10**6);
    let max_x = new BN(5*10**6);
    let max_y = new BN(5*10**6);
    const tx_deposit = await program.methods.deposit(seed_config,amount_deposit,max_x,max_y).accountsPartial({
      user: user.publicKey,
      mintX: mint_x,
      mintY: mint_y,
      mintLp: mint_lp,
      config: config_PDA,
      userX: user_ata_x,
      userY: user_ata_y,
      vaultX: vault_x_ata,
      vaultY: vault_y_ata,
      userLp: user_ata_lp
    }).signers([user]).rpc();
    console.log("Deposited.. Transaction signature: ", tx_deposit);

    const vault_x_ata_balance_before_swap = await connection.getTokenAccountBalance(vault_x_ata);
    console.log("Vault_X ATA Balance After Second Deposit: ",vault_x_ata_balance_before_swap.value.amount);

    const vault_y_ata_balance_before_swap = await connection.getTokenAccountBalance(vault_y_ata);
    console.log("Vault_Y ATA Balance Second Deposit : ",vault_y_ata_balance_before_swap.value.amount);

    const tx = await program.methods.swap(seed_config,is_x,amount,min).accountsPartial({
      user: user.publicKey,
      mintX: mint_x,
      mintY: mint_y,
      mintLp: mint_lp,
      config: config_PDA,
      userX: user_ata_x,
      userY: user_ata_y,
      vaultX: vault_x_ata,
      vaultY: vault_y_ata,
      userLp: user_ata_lp
    }).signers([user]).rpc();
   
    console.log("Swap succesful: ", tx);

  })

  //WORKING..
  it("Lock",async() => {
    const tx = await program.methods.lock(seed_config).accountsPartial({
      user: authority.publicKey,
      config: config_PDA
    }).signers([authority]).rpc();

    const configState = await program.account.config.fetch(config_PDA);
    expect(configState.locked).to.equal(true);

    console.log("The pool is locked succesfully. The signature: ",tx);
  })

  //WORKING...
  it("Unlock", async() => {
    const tx = await program.methods.unlock(seed_config).accountsPartial({
      user: authority.publicKey,
      config: config_PDA
    }).signers([authority]).rpc();

     const configState = await program.account.config.fetch(config_PDA);
    expect(configState.locked).to.equal(false);
    console.log("Pool Unlocked succesfully: ",tx);

  })
  
});
