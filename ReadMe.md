solana-escrow

### Build program 
```bash
cd program && cargo build-bpf
```
   
### Listen for logs on the address
```bash
solana logs | grep "[ADDRESS] invoke" -A 25
``` 


### Deploy program
```
solana program deploy target/deploy/solana_escrow.so --url https://api.devnet.solana.com --keypair ../client/keys/id.json
```