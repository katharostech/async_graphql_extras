use std::convert::Infallible;

use async_graphql::*;
use warp::Filter;

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

struct Query;

/// Output object to respond with user
#[derive(SimpleObject)]
pub struct UserData {
    username: String,
    display_name: String,
}

/// Input object to set user
#[derive(InputObject)]
pub struct UserDataInput {
    username: String,
    display_name: String,
}

#[Object]
impl Query {
    /// Ping endpoint that returns the same object as the input
    async fn ping(&self, user_input: UserDataInput) -> UserData {
        UserData {
            username: user_input.username,
            display_name: user_input.display_name,
        }
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
