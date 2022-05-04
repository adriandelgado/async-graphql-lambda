use crate::starwars::{QueryRoot, StarWars, StarWarsSchema};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Schema,
};
use async_graphql_lambda::{try_request, GraphQLBatchRequest};
use lambda_http::{
    http::header::CONTENT_TYPE, run, service_fn, Body, Error, IntoResponse, Request, Response,
};

mod starwars;

async fn function_handler(event: Request, schema: StarWarsSchema) -> Result<Response<Body>, Error> {
    if event.uri().path().ends_with("graphql") {
        let query: GraphQLBatchRequest = try_request!(event);

        Ok(query.execute(&schema).await.into_response())
    } else {
        Ok(Response::builder()
            .header(CONTENT_TYPE, "text/html; charset=utf-8")
            .body(playground_source(GraphQLPlaygroundConfig::new("/graphql")).into())?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let schema: StarWarsSchema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(StarWars::new())
        .finish();

    run(service_fn(|event| function_handler(event, schema.clone()))).await
}
