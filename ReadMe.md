solana-escrow

### Build program 
```bash
cd program && cargo build-bpf
```
   
### Start local node
```bash
solana-test-validator
``` 
   
### Listen for logs on the address
```bash
solana logs | grep "5nSKqqmeB3HEuPpqF5DJLN3pgUKoXwKQCa28KBs8DB9i invoke" -A 25
``` 


### Deploy program
```
solana program deploy target/deploy/solana_escrow.so --url http://api.devnet.solana.com --keypair ../client/keys/id.json
```