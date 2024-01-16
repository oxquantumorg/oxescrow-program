solana-escrow

### Build program 
```bash
cd program && cargo build-bpf
```
   
### Listen for logs on the address
```bash
solana logs | grep "[ADDRESS] invoke" -A 25
``` 


