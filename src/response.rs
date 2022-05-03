use lambda_http::{
    http::{self, HeaderValue},
    Body, IntoResponse, Response,
};

/// Responder for a GraphQL response.
///
/// This contains a batch response, but since regular responses are a type of batch response it
/// works for both.
#[derive(Debug)]
pub struct GraphQLResponse(pub async_graphql::BatchResponse);

impl From<async_graphql::Response> for GraphQLResponse {
    fn from(resp: async_graphql::Response) -> Self {
        Self(resp.into())
    }
}

impl From<async_graphql::BatchResponse> for GraphQLResponse {
    fn from(batch: async_graphql::BatchResponse) -> Self {
        Self(batch)
    }
}

impl IntoResponse for GraphQLResponse {
    fn into_response(self) -> Response<Body> {
        let body: Body = serde_json::to_string(&self.0)
            .expect("BatchResponse is serializable")
            .into();

        let mut response = Response::new(body);

        response.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        if self.0.is_ok() {
            if let Some(cache_control) = self
                .0
                .cache_control()
                .value()
                .and_then(|cache_control| cache_control.try_into().ok())
            {
                response
                    .headers_mut()
                    .insert(http::header::CACHE_CONTROL, cache_control);
            }
        }

        response.headers_mut().extend(self.0.http_headers());

        response
    }
}
