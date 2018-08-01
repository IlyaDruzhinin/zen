#ifndef CARDANO_RUST_H
# define CARDANO_RUST_H
/* Basic Types */

typedef int cardano_result;


/*********/
/* BIP39 */
/*********/

cardano_result cardano_bip39_encode(const unsigned char * const entropy_raw, unsigned long entropy_size, unsigned short *mnemonic_index, unsigned long mnemonic_size);

/***********/
/* Wallet  */
/***********/

typedef struct cardano_wallet cardano_wallet;
typedef struct cardano_account cardano_account;

cardano_wallet *cardano_wallet_new_from_seed(const unsigned char * const seed_ptr, unsigned int protocol_magic);
void cardano_wallet_delete(cardano_wallet *);

cardano_account *cardano_account_create(cardano_wallet *wallet, const unsigned char *alias, unsigned int index);
void cardano_account_delete(cardano_account *account);

unsigned long cardano_account_generate_addresses(cardano_account *account, int internal, unsigned int from_index, unsigned long num_indices, char *addresses_ptr[]);

#endif
