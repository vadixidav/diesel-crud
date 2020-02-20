# diesel-crud

[![Crates.io][ci]][cl] ![MIT/Apache][li] [![docs.rs][di]][dl] ![LoC][lo]

[ci]: https://img.shields.io/crates/v/diesel-crud.svg
[cl]: https://crates.io/crates/diesel-crud/

[li]: https://img.shields.io/crates/l/specs.svg?maxAge=2592000

[di]: https://docs.rs/diesel-crud/badge.svg
[dl]: https://docs.rs/diesel-crud/

[lo]: https://tokei.rs/b1/github/vadixidav/diesel-crud?category=code

Perscriptive API that makes it trivial implement simple CRUD operations with Diesel using Rust traits and auto-manages the connection pool

This crate is in the early stages and will be modified as more functionality is required from the ground-up.

## Example

The following could be your crate for specifying your API at the type level. You then just need REST endpoints
that take JSON that serializes into these types that then updates the database appropriately.

```rust
#[macro_use]
extern crate diesel;

pub mod schema;

use diesel::{Insertable, Queryable};
use diesel_crud::{Create, Load};
use schema::*;
use serde::{Deserialize, Serialize};

type Conn = diesel::pg::PgConnection;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct CreateUser {
    pub username: String,
}

impl Create for CreateUser {
    type Table = users::table;

    fn table() -> Self::Table {
        users::table
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
}

struct GetUsers;

impl Load<Conn> for GetUsers {
    type Item = User;
    type Query = users::table;

    fn query(self) -> Self::Query {
        users::table
    }
}
```
