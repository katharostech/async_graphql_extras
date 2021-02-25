# async_graphql_extras

[![Crates.io](https://img.shields.io/crates/v/async_graphql_extras.svg)](https://crates.io/crates/async_graphql_extras)
[![Docs.rs](https://docs.rs/async_graphql_extras/badge.svg)](https://docs.rs/async_graphql_extras)
[![Katharos License](https://img.shields.io/badge/License-Katharos-blue)](https://github.com/katharostech/katharos-license)

Experimental helper macros for use with [`async_graphql`].

## Example
```rust no_run
use std::convert::Infallible;
use async_graphql::*;
use async_graphql_extras::graphql_object;
use warp::Filter;

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

struct Query;


/// Information about the user
##[graphql_object]
pub struct UserData {
    username: String,
    display_name: String,
}

##[Object]
impl Query {
    /// Ping endpoint that returns the same object as the input
    async fn ping(&self, user_input: UserDataInput) -> UserData {
        UserData {
            username: user_input.username,
            display_name: user_input.display_name,
        }
    }
}

##[tokio::main]
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

[`Read`]: https://doc.rust-lang.org/stable/std/io/trait.Read.html
[`Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html
