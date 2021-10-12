/* tslint:disable */
/* eslint-disable */
/**
* generate proof-of-work
* ```rust
* fn main() {
*    use mcaptcha_browser::*;
*    use pow_sha256::*;
*
*
*    // salt using which PoW should be computed
*    const SALT: &str = "yrandomsaltisnotlongenoug";
*    // one-time phrase over which PoW should be computed
*    const PHRASE: &str = "ironmansucks";
*    // and the difficulty factor
*    const DIFFICULTY: u32 = 1000;
*
*    // currently gen_pow() returns a JSON formated string to better communicate
*    // with JavaScript. See [PoW<T>][pow_sha256::PoW] for schema
*    let serialised_work = gen_pow(SALT.into(), PHRASE.into(), DIFFICULTY);
*
*
*    let work: Work = serde_json::from_str(&serialised_work).unwrap();
*    
*    let work = PoWBuilder::default()
*        .result(work.result)
*        .nonce(work.nonce)
*        .build()
*        .unwrap();
*    
*    let config = ConfigBuilder::default().salt(SALT.into()).build().unwrap();
*    assert!(config.is_valid_proof(&work, &PHRASE.to_string()));
*    assert!(config.is_sufficient_difficulty(&work, DIFFICULTY));
* }
* ```
* @param {string} salt
* @param {string} phrase
* @param {number} difficulty_factor
* @returns {string}
*/
export function gen_pow(salt: string, phrase: string, difficulty_factor: number): string;
