use serde_json::Value;

async fn send_request(did: &str, mnemonic: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let url = format!("http://localhost:5000/authenticate?did={}&mnemonic={}", did, mnemonic);

    let response = reqwest::get(&url)
        .await?
        .json::<Value>()
        .await?;

    Ok(response)
}

#[tokio::main]
async fn main() {
    let did = "did:sam:root:5DRRAK6uVDwnWX729Y3WzSurvqEFXDmAF1HXw6LFTokh7Bjc";
    let mnemonic = "tonight~hurdle~price~naive~brief~slogan~immune~current~abandon~supreme~limb~magnet";

    match send_request(did, mnemonic).await {
        Ok(response) => {
            if response["error"] == Value::Bool(false) {
                if response["data"]["exists"] == Value::Bool(true) {
                    println!("The DID exists onchain");
                } else {
                    println!("DID not recognized");
                }
            } else {
                println!("An internal server error has occured");
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }
}