# notion-to-jarkup

Convert Notion blocks into [jarkup](https://github.com/46ki75/jarkup) JSON.

## Example

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let notion_api_key = std::env::var("NOTION_API_KEY")?;
    let block_id = std::env::var("BLOCK_ID")?;

    let notionrs_client = notionrs::client::Client::new().secret(notion_api_key);
    let reqwest_client = reqwest::Client::new();

    let client = notion_to_jarkup::client::Client {
        notionrs_client,
        reqwest_client,
    };

    let result = client.convert_block(&block_id).await?;

    println!("{}", serde_json::to_string(&result).unwrap());

    Ok(())
}
```
