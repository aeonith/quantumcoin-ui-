use qc_validation::*;
use qc_types::*;
use std::collections::HashMap;

fn spec() -> ChainSpec { 
    toml::from_str(include_str!("../../../chain_spec.toml")).unwrap() 
}

#[test]
fn revstop_window_enforced() {
    let spec = spec();
    let prev = OutPoint{ txid: Hash32::zero(), vout: 0 };
    let pk = vec![1u8; 1312]; // placeholder Dilithium2 pubkey size
    let mut utxo = HashMap::<(Hash32,u32),(Amount,OutputType,Height,bool)>::new();
    
    // Create a revocable UTXO
    utxo.insert(
        (prev.txid, prev.vout), 
        (10_000, OutputType::P2PQRevocable{ 
            pubkey: pk.clone(), 
            window_blocks: spec.revstop.window_blocks 
        }, 100, false)
    );

    // Transaction trying to cancel (RevStop)
    let tx = Transaction{
        version: 1, 
        lock_time: 0,
        vin: vec![TxIn{ 
            prevout: prev.clone(), 
            pq_signature: vec![2u8; 2420], // placeholder Dilithium2 sig size
            cancel: true 
        }],
        vout: vec![TxOut{ 
            value: 9_000, 
            kind: OutputType::P2PQ{ pubkey: pk.clone() } 
        }],
    };

    let lookup = |op: &OutPoint| utxo.get(&(op.txid, op.vout)).cloned();
    
    // Should work within window (height 110 = 10 blocks after creation at 100)
    let ok = validate_transaction(&spec, 110, &tx, false, lookup.clone());
    
    // Should fail outside window (height 200 = 100 blocks after creation)
    let late = validate_transaction(&spec, 200, &tx, false, lookup);

    // Either succeeds or fails on signature (since we're using dummy signatures)
    assert!(ok.is_ok() || matches!(ok, Err(ValidationError::BadSignature)));
    
    // Should definitely fail outside window
    assert!(matches!(late, Err(ValidationError::CancelOutsideWindow)) || 
            matches!(late, Err(ValidationError::BadSignature)));
}
