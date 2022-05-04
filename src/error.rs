use lambda_http::{
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    Body, IntoResponse, Response,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphQLError {
    #[error("empty body")]
    EmptyBody,
    #[error("error while reading query")]
    QueryError,
    #[error("only GET and POST requests are allowed")]
    MethodNotAllowed,
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    ParseError(#[from] async_graphql::ParseRequestError),
}

impl GraphQLError {
    fn error_type(&self) -> &'static str {
        match self {
            GraphQLError::EmptyBody => "EmptyBody",
            GraphQLError::QueryError => "QueryError",
            GraphQLError::MethodNotAllowed => "MethodNotAllowed",
            GraphQLError::JsonError(e) => type_name_of_val(e),
            GraphQLError::ParseError(e) => type_name_of_val(e),
        }
    }

    fn to_body(&self) -> Body {
        serde_json::to_string(&LambdaError {
            error_type: self.error_type(),
            error_message: self.to_string(),
        })
        .unwrap()
        .into()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LambdaError {
    error_type: &'static str,
    error_message: String,
}

impl IntoResponse for GraphQLError {
    fn into_response(self) -> lambda_http::Response<Body> {
        let mut response = Response::new(self.to_body());
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        match self {
            GraphQLError::MethodNotAllowed => {
                *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            }
            _ => *response.status_mut() = StatusCode::BAD_REQUEST,
        };
        response
    }
}

fn type_name_of_val<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}
