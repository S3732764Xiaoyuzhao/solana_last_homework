use borsh::{BorshDeserialize, BorshSerialize};
use last_homework::{entrypoint::process_instruction, state::UserBalance};
use solana_program_test::*;
use solana_program::{system_instruction};
use solana_sdk::{account::{Account}, instruction::{AccountMeta, Instruction}, pubkey::Pubkey, signature::Signer, signer::keypair::Keypair,
    sysvar::rent::Rent, transaction::Transaction};
use std::mem;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

#[tokio::test]

async fn test_helloworld() {
    let program_id = Pubkey::new_unique();
    let user_keypair = Keypair::new();
    let user_pubkey = user_keypair.pubkey();
    let spl_token_authority_keypair = Keypair::new();
    let spl_token_authority_pubkey = spl_token_authority_keypair.pubkey();
    let spl_token_address = Keypair::new();
    let token_program_id =
        Pubkey::from_str(&"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    let spl_token_address_pubkey = spl_token_address.pubkey();
    let user_ada =
        get_associated_token_address(&user_pubkey, &spl_token_address_pubkey);

    let mut program_test = ProgramTest::new(
        "last_homework", // Run the BPF version with `cargo test-bpf`
        program_id,
        processor!(process_instruction), // Run the native version with `cargo test`
    );
    //
    let user_pubkey_string = user_pubkey.to_string() ;
    let digest = md5::compute(user_pubkey_string);
    let seed = digest.iter().map(|&x|{
        [char::from_digit((x/16) as u32 ,16).unwrap(), 
        char::from_digit((x%16) as u32 ,16).unwrap()]
        .iter().map(|&a|{a.to_string()}).collect::<String>()
    }).collect::<String>();
    let user_derived_pubkey = Pubkey::create_with_seed(&user_pubkey,
        &seed, &program_id).unwrap();

    program_test.add_account(
        user_derived_pubkey,
        Account {
            lamports: Rent::default().minimum_balance(mem::size_of::<u64>()),
            data: vec![0_u8; mem::size_of::<u64>()],
            owner: program_id,
            ..Account::default()
        },
    );
    //
    let (program_derived_address, nonce) =
    Pubkey::find_program_address(&[b"last_homework"], &program_id);
    program_test.add_account(
        program_derived_address,
        Account {
            lamports: 100,
            owner: program_id,
            ..Account::default()
        },
    );
    let program_derived_address_ada =
    get_associated_token_address(&program_derived_address, &spl_token_address_pubkey);

    //
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let user_derived_pubkey_acc = banks_client.get_account(user_derived_pubkey).await.unwrap().unwrap();
    let user_derived_pubkey_acc_deserialize: UserBalance =
        BorshDeserialize::deserialize(&mut &user_derived_pubkey_acc.data[..]).unwrap();
    assert_eq!(user_derived_pubkey_acc_deserialize.balance, 0);

    //
    println!("mint start");
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(82);
    let mut transaction_create_mint = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &spl_token_address_pubkey,
                mint_rent,
                82,
                &token_program_id,
            ),
            spl_token::instruction::initialize_mint(
                &token_program_id,
                &spl_token_address_pubkey,
                &spl_token_authority_pubkey,
                None,
                0,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    transaction_create_mint.sign(
        &[
            &payer,
            &spl_token_address,
        ],
        recent_blockhash,
    );
    banks_client
        .process_transaction(transaction_create_mint)
        .await
        .unwrap();
    println!("mint success");

    println!("create ada start");
    let mut transaction_create_mint3 = Transaction::new_with_payer(
        &[
            spl_associated_token_account::create_associated_token_account(
                &payer.pubkey(),
                &user_pubkey,
                &spl_token_address_pubkey,
            ),
            spl_associated_token_account::create_associated_token_account(
                &payer.pubkey(),
                &program_derived_address,
                &spl_token_address_pubkey,
            ),
        ],
        Some(&payer.pubkey()),
    );
    transaction_create_mint3.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_create_mint3)
        .await
        .unwrap();
    println!("create ada success");

    println!("mint user start");
    let mut transaction_create_mint2 = Transaction::new_with_payer(
        &[
            spl_token::instruction::mint_to_checked(
                &token_program_id,
                &spl_token_address_pubkey,
                &user_ada,
                &spl_token_authority_pubkey,
                &[],
                100,
                0,
            )
            .unwrap(),
            spl_token::instruction::mint_to_checked(
                &token_program_id,
                &spl_token_address_pubkey,
                &program_derived_address_ada,
                &spl_token_authority_pubkey,
                &[],
                100,
                0,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    transaction_create_mint2.sign(&[&payer, &spl_token_authority_keypair], recent_blockhash);
    banks_client
        .process_transaction(transaction_create_mint2)
        .await
        .unwrap();
    println!("mint user success");


    println!("user deposit start");
    let user_derived_pubkey_acc = banks_client.get_account(user_derived_pubkey).await.unwrap().unwrap();
    let user_derived_pubkey_acc_data: UserBalance =
    BorshDeserialize::deserialize(&mut &user_derived_pubkey_acc.data[..]).unwrap();
    assert_eq!(user_derived_pubkey_acc_data.balance, 0);
    let amount = 10u64;
    let mut data = vec![0u8];
    data.extend(amount.try_to_vec().unwrap());
    let data: &[u8] = data.as_slice();
    println!("{:?}",data);
    let mut transaction_submit = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[0,10u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8],
            vec![
                AccountMeta::new(user_keypair.pubkey(), true), //用户
                AccountMeta::new(user_ada, false),       //用户关联账号
                AccountMeta::new(user_derived_pubkey, false),    //用户派生地址
                AccountMeta::new(program_derived_address, false),          //program派生地址
                AccountMeta::new(program_derived_address_ada, false),    //program spl ada
                AccountMeta::new(token_program_id, false),
                AccountMeta::new(spl_token_address_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction_submit.sign(&[&payer, &user_keypair], recent_blockhash);
    banks_client
        .process_transaction(transaction_submit)
        .await
        .unwrap();
    let user_derived_pubkey_acc = banks_client.get_account(user_derived_pubkey).await.unwrap().unwrap();    
    let user_derived_pubkey_acc_data: UserBalance =
        BorshDeserialize::deserialize(&mut &user_derived_pubkey_acc.data[..]).unwrap();
    assert_eq!(user_derived_pubkey_acc_data.balance, 10);
    println!("user deposit success");

    println!("user withdraw start");
    let user_derived_pubkey_acc = banks_client.get_account(user_derived_pubkey).await.unwrap().unwrap();
    let user_derived_pubkey_acc_data: UserBalance =
    BorshDeserialize::deserialize(&mut &user_derived_pubkey_acc.data[..]).unwrap();
    assert_eq!(user_derived_pubkey_acc_data.balance, 10);
    let amount = 10u64;
    let mut data = vec![0u8];
    data.extend(amount.try_to_vec().unwrap());
    let data: &[u8] = data.as_slice();
    println!("{:?}",data);
    let mut transaction_submit = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &[1,nonce],
            vec![
                AccountMeta::new(user_keypair.pubkey(), true), //用户
                AccountMeta::new(user_ada, false),       //用户关联账号
                AccountMeta::new(user_derived_pubkey, false),    //用户派生地址
                AccountMeta::new(program_derived_address, false),          //program派生地址
                AccountMeta::new(program_derived_address_ada, false),    //program 募集币spl ada
                AccountMeta::new(token_program_id, false),
                AccountMeta::new(spl_token_address_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction_submit.sign(&[&payer, &user_keypair], recent_blockhash);
    banks_client
        .process_transaction(transaction_submit)
        .await
        .unwrap();
    let user_derived_pubkey_acc = banks_client.get_account(user_derived_pubkey).await.unwrap().unwrap();    
    let user_derived_pubkey_acc_data: UserBalance =
        BorshDeserialize::deserialize(&mut &user_derived_pubkey_acc.data[..]).unwrap();
    assert_eq!(user_derived_pubkey_acc_data.balance, 0);
    println!("user withdraw success");

}
