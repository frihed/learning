use tide::Request;
use serde::Deserialize;
use serde::Serialize;
use log::*;
use anyhow::Result;
use tide::sse;

#[derive(Debug, Deserialize, Serialize)]
struct Animal {
    name: String,
    legs: u8,
}

pub async fn start() -> Result<()>{
    let mut app = tide::new();
    app.at("/orders/shoes").post(order_shoes);
    app.at("/orders/shoes").get(order_example);
    app.at("/sse").get(sse::endpoint(|_req, sender|async move {
        sender.send("fruit", "banana", None).await?;
        sender.send("fruit", "apple", None).await?;
        Ok(())
    }));
    app.listen("127.0.0.1:8010").await?;

    Ok(())
}

async fn order_example(mut _req: Request<()>) -> tide::Result {
    info!("get order example!");
    let ani = Animal {
        name: "cat".to_string(),
        legs: 4,
    };
    Ok(serde_json::to_string(&ani).unwrap().into())
}

async fn order_shoes(mut req: Request<()>) -> tide::Result {
    info!("post order example");

    let Animal {name, legs} = req.body_json().await?;

    Ok(format!("Hello, {}! I've put in an order for {} shoes", name, legs).into())
}