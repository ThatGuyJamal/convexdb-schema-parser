use convex::ConvexClient;

const CONVEX_URL: &str = "https://notable-orca-705.convex.cloud";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    // let client = ConvexClient::new(CONVEX_URL).await?;

    println!("Hello, world!");

    Ok(())
}