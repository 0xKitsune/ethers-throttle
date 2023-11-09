use ethers::{
    providers::{Http, Middleware, Provider},
    types::BlockNumber,
};
use ethers_throttle::ThrottledJsonRpcClient;
use std::sync::Arc;
use url::Url;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize the json rpc client
    let rpc_endpoint = std::env::var("ETHEREUM_RPC_ENDPOINT")?;
    let http_provider = Http::new(Url::parse(&rpc_endpoint)?);

    let requests_per_second = 5;
    let throttled_http_provider =
        ThrottledJsonRpcClient::new(http_provider, requests_per_second, None);

    let middleware = Arc::new(Provider::new(throttled_http_provider));

    let block = middleware.get_block_with_txs(BlockNumber::Latest).await?;

    if let Some(block) = block {
        for tx in block.transactions {
            let tx = middleware.get_transaction(tx.hash).await?;

            if let Some(tx) = tx {
                println!("Transaction: {:?}", tx);
            }
        }
    }

    Ok(())
}
