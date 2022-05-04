use crate::{error::GraphQLError, GraphQLResponse};
use async_graphql::{ObjectType, Schema, SubscriptionType};
use lambda_http::{
    ext::RequestExt, http::Method, request::RequestContext, Body, Request as LambdaRequest,
};

/// Extractor for GraphQL request.
#[derive(Debug)]
pub struct GraphQLRequest(pub async_graphql::Request);

impl GraphQLRequest {
    /// Unwraps the value to `async_graphql::Request`.
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> async_graphql::Request {
        self.0
    }

    /// Shortcut method to execute the request on the schema.
    pub async fn execute<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> GraphQLResponse
    where
        Query: ObjectType + 'static,
        Mutation: ObjectType + 'static,
        Subscription: SubscriptionType + 'static,
    {
        GraphQLResponse(schema.execute(self.into_inner()).await.into())
    }
}

/// Extractor for GraphQL batch request.
#[derive(Debug)]
pub struct GraphQLBatchRequest(pub async_graphql::BatchRequest);

impl GraphQLBatchRequest {
    /// Unwraps the value to `async_graphql::BatchRequest`.
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> async_graphql::BatchRequest {
        self.0
    }

    /// Shortcut method to execute the request on the schema.
    pub async fn execute<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> GraphQLResponse
    where
        Query: ObjectType + 'static,
        Mutation: ObjectType + 'static,
        Subscription: SubscriptionType + 'static,
    {
        GraphQLResponse(schema.execute_batch(self.into_inner()).await)
    }
}

impl TryFrom<LambdaRequest> for GraphQLRequest {
    type Error = GraphQLError;

    fn try_from(req: LambdaRequest) -> Result<Self, Self::Error> {
        Ok(Self(
            GraphQLBatchRequest::try_from(req)?
                .into_inner()
                .into_single()?,
        ))
    }
}

impl TryFrom<LambdaRequest> for GraphQLBatchRequest {
    type Error = GraphQLError;

    fn try_from(request: LambdaRequest) -> Result<Self, Self::Error> {
        match (request.method(), request.body()) {
            (&Method::GET, _) => {
                let req = query_to_request(&request)?;
                Ok(Self(async_graphql::BatchRequest::Single(req)))
            }
            (&Method::POST, Body::Empty) => Err(GraphQLError::EmptyBody),
            (&Method::POST, Body::Text(text)) => {
                serde_json::from_str::<async_graphql::BatchRequest>(text)
                    .map_err(GraphQLError::JsonError)
                    .map(Self)
            }
            (&Method::POST, Body::Binary(binary)) => {
                serde_json::from_slice::<async_graphql::BatchRequest>(binary)
                    .map_err(GraphQLError::JsonError)
                    .map(Self)
            }
            _ => Err(GraphQLError::MethodNotAllowed),
        }
    }
}

fn query_to_request(req: &LambdaRequest) -> Result<async_graphql::Request, GraphQLError> {
    let query_map = req.query_string_parameters();
    let query = match req.request_context() {
        // API Gateway Payload Version 2.0 doesn't follow spec.
        // See https://github.com/calavera/query-map-rs/issues/1 and
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html
        RequestContext::ApiGatewayV2(_) => query_map.all("query").map(|x| x.join(",")),
        _ => query_map.first("query").map(String::from),
    }
    .ok_or(GraphQLError::QueryError)?;

    let mut request = async_graphql::Request::new(query);

    if let Some(operation_name) = query_map.first("operationName") {
        request = request.operation_name(operation_name);
    }

    if let Some(variables) = query_map.first("variables") {
        let value = serde_json::from_str(variables).unwrap_or_default();
        let variables = async_graphql::Variables::from_json(value);
        request = request.variables(variables);
    }

    Ok(request)
}
