## 前置需要条件

```ts
spl-token地址
用户派生地址：md5（用户公钥）得到16位字节数组，然后根据16进制分解成32位字符串
用户关联账号
program派生地址：seed为last_homework
program派生地址对spl-token的关联账号
```

## deposit

```ts
keys：[
  { pubkey: 用户, 需要签名},
  { pubkey: 用户关联账号},
  { pubkey: 用户派生地址},
  { pubkey: program派生地址},
  { pubkey: program spl ada},
  { pubkey: token_program_id},//TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
  { pubkey: spl_token账号},
]，
programId: programId,
data: Buffer.from(Uint8Array.of(0, amount))//0:deposit amount:u64

签名账户:[payer, 管理员最高权限账号密钥对]
```

## withdraw

```ts
keys：[
  { pubkey: 用户, 需要签名},
  { pubkey: 用户关联账号},
  { pubkey: 用户派生地址},
  { pubkey: program派生地址},
  { pubkey: program spl ada},
  { pubkey: token_program_id},
  { pubkey: spl_token账号},
]，
programId: programId,
data: Buffer.from(Uint8Array.of(1, nonce))//0:deposit nonce:u8(find_program_address得到的随机数)

签名账户:[payer, 管理员最高权限账号密钥对]
```
