# async_graphql_extras

[![Crates.io](https://img.shields.io/crates/v/async_graphql_extras.svg)](https://crates.io/crates/async_graphql_extras)
[![Docs.rs](https://docs.rs/async_graphql_extras/badge.svg)](https://docs.rs/async_graphql_extras)
[![Katharos License](https://img.shields.io/badge/License-Katharos-blue)](https://github.com/katharostech/katharos-license)

Experimental helper macros for use with [`async_graphql`].

## Example
```rust
use std::convert::Infallible;
use async_graphql::*;
use async_graphql_extras::graphql_object;
use warp::Filter;

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

struct Query;


/// Information about the user
#[graphql_object]
pub struct UserData {
    username: String,
    display_name: String,
    // You can set a custom type to use for fields in the input mode
    #[graphql_object(input_type = "InputFavorites")]
    favorites: Favorites
}

#[graphql_object(
    // You can override the default input type name
    input_type_name="InputFavorites"
)]
pub struct Favorites {
    food: String,
}

#[Object]
impl Query {
    /// Ping endpoint that returns the same object as the input
    // here the `user_input` arg has type `UserDataInput` which is the
    // corresponding input type to `UserData` which was automatically
    // generated.
    async fn ping(&self, user_input: UserDataInput) -> UserData {
        user_input.into()
    }
}

#[tokio::main]
async fn main() {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let filter = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (MySchema, async_graphql::Request)| async move {
            // Execute query
            let resp = schema.execute(request).await;

            // Return result
            Ok::<_, Infallible>(async_graphql_warp::Response::from(resp))
        },
    );
    warp::serve(filter).run(([0, 0, 0, 0], 8000)).await;
}
```

[`async_graphql`]: https://docs.rs/async_graphql
