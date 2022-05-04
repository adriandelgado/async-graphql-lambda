//! Async-graphql integration with Axum
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)]

/// Macro for converting a `lambda_http::Request`, into a `GraphQLRequest` or
/// `GraphQLBatchRequest` which returns early in the case of errors. The difference
/// between this macro with the `?` operator is that the early return is an
/// `Ok(http::Response)` with better formatting than just throwing an error.
#[macro_export]
macro_rules! try_request {
    ($request:expr $(,)?) => {
        match ::core::convert::TryInto::try_into($request) {
            ::core::result::Result::Ok(q) => q,
            ::core::result::Result::Err(e) => {
                return Ok(::lambda_http::IntoResponse::into_response(e))
            }
        }
    };
}

mod error;
mod request;
mod response;

pub use request::{GraphQLBatchRequest, GraphQLRequest};
pub use response::GraphQLResponse;
