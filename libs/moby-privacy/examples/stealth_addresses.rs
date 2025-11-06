use moby_privacy::*;
use moby_types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Stealth Addresses Example");

    // Generate stealth key pair for recipient
    let recipient_keypair = stealth::StealthKeyPair::generate();

    println!("Generated stealth keypair for recipient");
    println!("Public spend key: {}", hex::encode(recipient_keypair.public_spend.to_bytes()));
    println!("Public view key: {}", hex::encode(recipient_keypair.public_view.to_bytes()));

    // Sender derives a stealth address for the recipient
    let (stealth_address, ephemeral_key) = recipient_keypair.derive_stealth_address(
        &recipient_keypair.public_spend,
        &recipient_keypair.public_view,
    )?;

    println!("\nDerived stealth address:");
    println!("Address: {}", stealth_address.to_string());
    println!("Ephemeral key: {}", hex::encode(ephemeral_key.to_bytes()));

    // Recipient scans for payments
    let payment_found = recipient_keypair.check_stealth_payment(
        &stealth_address,
        &ephemeral_key,
    )?;

    if payment_found {
        println!("\n✅ Recipient successfully detected stealth payment!");

        // Recipient can spend the stealth payment
        let spend_key = recipient_keypair.derive_spend_key(
            &stealth_address,
            &ephemeral_key,
        )?;

        println!("Derived spend key: {}", hex::encode(spend_key.to_bytes()));
    } else {
        println!("\n❌ Payment not detected");
    }

    // Generate multiple stealth addresses
    println!("\nGenerating multiple stealth addresses:");
    for i in 1..=3 {
        let (addr, _) = recipient_keypair.derive_stealth_address(
            &recipient_keypair.public_spend,
            &recipient_keypair.public_view,
        )?;
        println!("Address {}: {}", i, addr.to_string());
    }

    Ok(())
}