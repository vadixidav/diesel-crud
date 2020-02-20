use diesel::{
    backend::UsesAnsiSavepointSyntax,
    insertable::CanInsertInSingleQuery,
    query_builder::{InsertStatement, QueryFragment},
    query_dsl::methods::{ExecuteDsl, LoadQuery},
    r2d2::{ConnectionManager, PooledConnection},
    Insertable, RunQueryDsl,
};
use dotenv::dotenv;
use failure::Fail;
use std::env;

#[derive(Debug, Fail, derive_more::From)]
pub enum Error {
    #[fail(display = "database query error: {}", 0)]
    Database(diesel::result::Error),
    #[fail(display = "thread pool connection error: {}", 0)]
    Connection(r2d2::Error),
}

type Pool<BaseConnection> = diesel::r2d2::Pool<ConnectionManager<BaseConnection>>;
type Connection<BaseConnection> = PooledConnection<ConnectionManager<BaseConnection>>;

pub struct Db<BaseConnection: diesel::connection::Connection + 'static> {
    pool: Pool<BaseConnection>,
}

impl<BaseConnection> Db<BaseConnection>
where
    BaseConnection: diesel::connection::Connection + 'static,
{
    /// Creates a database connection pool.
    pub fn new() -> Result<Self, Error> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Ok(Self {
            pool: Pool::<BaseConnection>::builder().build(ConnectionManager::new(&database_url))?,
        })
    }

    pub fn conn(&self) -> Result<Connection<BaseConnection>, Error> {
        Ok(self.pool.get()?)
    }
}

impl<BaseConnection> Db<BaseConnection>
where
    BaseConnection: diesel::connection::Connection<
            TransactionManager = diesel::connection::AnsiTransactionManager,
        > + 'static,
    BaseConnection::Backend: UsesAnsiSavepointSyntax,
{
    pub fn cud<C>(&self, cud: C) -> Result<(), Error>
    where
        C: Cud<BaseConnection>,
        C::Query: QueryFragment<BaseConnection::Backend> + diesel::query_builder::QueryId,
    {
        cud.execute(self)
    }

    pub fn load<L>(&self, load: L) -> Result<Vec<L::Item>, Error>
    where
        L: Load<BaseConnection>,
    {
        load.load(self)
    }
}

/// Trait which is implemented by create, update, and delete operations.
pub trait Cud<BaseConnection>: Sized
where
    BaseConnection: diesel::connection::Connection<
            TransactionManager = diesel::connection::AnsiTransactionManager,
        > + 'static,
    BaseConnection::Backend: UsesAnsiSavepointSyntax,
{
    type Query: RunQueryDsl<Connection<BaseConnection>> + ExecuteDsl<Connection<BaseConnection>>;

    fn execute(self, db: &Db<BaseConnection>) -> Result<(), Error> {
        Ok(self.query().execute(&db.conn()?).map(|_| ())?)
    }

    fn query(self) -> Self::Query;
}

/// Trait for update operations which auto-implements [`Cud`].
pub trait Create: Sized {
    type Table: diesel::Table;

    fn table() -> Self::Table;
}

impl<T, Table, BaseConnection> Cud<BaseConnection> for T
where
    T: Create<Table = Table> + Insertable<Table>,
    Table: diesel::Table,
    T::Values: CanInsertInSingleQuery<BaseConnection::Backend>,
    T::Values: QueryFragment<BaseConnection::Backend>,
    Table::FromClause: QueryFragment<BaseConnection::Backend>,
    BaseConnection: diesel::connection::Connection<
            TransactionManager = diesel::connection::AnsiTransactionManager,
        > + 'static,
    BaseConnection::Backend: UsesAnsiSavepointSyntax,
{
    type Query = InsertStatement<Table, <Self as Insertable<Table>>::Values>;

    fn query(self) -> Self::Query {
        diesel::insert_into(T::table()).values(self)
    }
}

/// Trait which is implemented by read operations.
pub trait Load<BaseConnection>: Sized
where
    BaseConnection: diesel::connection::Connection + 'static,
{
    type Item;
    type Query: LoadQuery<Connection<BaseConnection>, Self::Item>;

    fn load(self, db: &Db<BaseConnection>) -> Result<Vec<Self::Item>, Error> {
        Ok(self.query().load::<Self::Item>(&db.conn()?)?)
    }

    fn query(self) -> Self::Query;
}
