use std::convert::Infallible;

use async_graphql::*;
use async_graphql_extras::graphql_object;
use warp::Filter;

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

struct Query;

/// Information about the user
#[graphql_object(
    // Custom doc string for generated `UserDataInput` struct
    input_object_doc = "Input for user information data",
)]
pub struct UserData {
    username: String,
    display_name: String,

    // Indicate that this is also a `graphql_object`
    #[graphql_object(nested)]
    favorites: UserFavorites,
}

/// A user's favorite stuff
#[graphql_object]
pub struct UserFavorites {
    food: String,
}

#[Object]
impl Query {
    /// Ping endpoint that returns the same object as the input
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
