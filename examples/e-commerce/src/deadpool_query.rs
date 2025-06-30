#[derive(Debug, Clone, Copy, postgres_types::ToSql, postgres_types::FromSql)]
#[postgres(name = "order_status")]
pub enum OrderStatus {
    #[postgres(name = "pending")]
    Pending,
    #[postgres(name = "processing")]
    Processing,
    #[postgres(name = "shipped")]
    Shipped,
    #[postgres(name = "delivered")]
    Delivered,
    #[postgres(name = "cancelled")]
    Cancelled,
}
pub struct CreateUserRow {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    pub full_name: Option<String>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}
impl CreateUserRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            username: row.try_get(1)?,
            email: row.try_get(2)?,
            hashed_password: row.try_get(3)?,
            full_name: row.try_get(4)?,
            created_at: row.try_get(5)?,
            updated_at: row.try_get(6)?,
        })
    }
}
pub struct CreateUser<'a> {
    username: &'a str,
    email: &'a str,
    hashed_password: &'a str,
    full_name: Option<&'a str>,
}
impl<'a> CreateUser<'a> {
    pub const QUERY: &'static str = r"INSERT INTO users (
    username, email, hashed_password, full_name
) VALUES (
    $1, $2, $3, $4
)
RETURNING id, username, email, hashed_password, full_name, created_at, updated_at";
    pub async fn query_one(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<CreateUserRow, deadpool_postgres::tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[
                    &self.username,
                    &self.email,
                    &self.hashed_password,
                    &self.full_name,
                ],
            )
            .await?;
        CreateUserRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Option<CreateUserRow>, deadpool_postgres::tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[
                    &self.username,
                    &self.email,
                    &self.hashed_password,
                    &self.full_name,
                ],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateUserRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateUser<'a> {
    pub const fn builder() -> CreateUserBuilder<'a, ((), (), (), ())> {
        CreateUserBuilder {
            fields: ((), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct CreateUserBuilder<'a, Fields = ((), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, Email, HashedPassword, FullName>
    CreateUserBuilder<'a, ((), Email, HashedPassword, FullName)>
{
    pub fn username(
        self,
        username: &'a str,
    ) -> CreateUserBuilder<'a, (&'a str, Email, HashedPassword, FullName)> {
        let ((), email, hashed_password, full_name) = self.fields;
        let _phantom = self._phantom;
        CreateUserBuilder {
            fields: (username, email, hashed_password, full_name),
            _phantom,
        }
    }
}
impl<'a, Username, HashedPassword, FullName>
    CreateUserBuilder<'a, (Username, (), HashedPassword, FullName)>
{
    pub fn email(
        self,
        email: &'a str,
    ) -> CreateUserBuilder<'a, (Username, &'a str, HashedPassword, FullName)> {
        let (username, (), hashed_password, full_name) = self.fields;
        let _phantom = self._phantom;
        CreateUserBuilder {
            fields: (username, email, hashed_password, full_name),
            _phantom,
        }
    }
}
impl<'a, Username, Email, FullName> CreateUserBuilder<'a, (Username, Email, (), FullName)> {
    pub fn hashed_password(
        self,
        hashed_password: &'a str,
    ) -> CreateUserBuilder<'a, (Username, Email, &'a str, FullName)> {
        let (username, email, (), full_name) = self.fields;
        let _phantom = self._phantom;
        CreateUserBuilder {
            fields: (username, email, hashed_password, full_name),
            _phantom,
        }
    }
}
impl<'a, Username, Email, HashedPassword>
    CreateUserBuilder<'a, (Username, Email, HashedPassword, ())>
{
    pub fn full_name(
        self,
        full_name: Option<&'a str>,
    ) -> CreateUserBuilder<'a, (Username, Email, HashedPassword, Option<&'a str>)> {
        let (username, email, hashed_password, ()) = self.fields;
        let _phantom = self._phantom;
        CreateUserBuilder {
            fields: (username, email, hashed_password, full_name),
            _phantom,
        }
    }
}
impl<'a> CreateUserBuilder<'a, (&'a str, &'a str, &'a str, Option<&'a str>)> {
    pub const fn build(self) -> CreateUser<'a> {
        let (username, email, hashed_password, full_name) = self.fields;
        CreateUser {
            username,
            email,
            hashed_password,
            full_name,
        }
    }
}
pub struct GetUserByEmailRow {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    pub full_name: Option<String>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}
impl GetUserByEmailRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            username: row.try_get(1)?,
            email: row.try_get(2)?,
            hashed_password: row.try_get(3)?,
            full_name: row.try_get(4)?,
            created_at: row.try_get(5)?,
            updated_at: row.try_get(6)?,
        })
    }
}
pub struct GetUserByEmail<'a> {
    email: &'a str,
}
impl<'a> GetUserByEmail<'a> {
    pub const QUERY: &'static str = r"SELECT id, username, email, hashed_password, full_name, created_at, updated_at FROM users
WHERE email = $1 LIMIT 1";
    pub async fn query_one(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<GetUserByEmailRow, deadpool_postgres::tokio_postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.email]).await?;
        GetUserByEmailRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Option<GetUserByEmailRow>, deadpool_postgres::tokio_postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.email]).await?;
        match row {
            Some(row) => Ok(Some(GetUserByEmailRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> GetUserByEmail<'a> {
    pub const fn builder() -> GetUserByEmailBuilder<'a, ((),)> {
        GetUserByEmailBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct GetUserByEmailBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetUserByEmailBuilder<'a, ((),)> {
    pub fn email(self, email: &'a str) -> GetUserByEmailBuilder<'a, (&'a str,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        GetUserByEmailBuilder {
            fields: (email,),
            _phantom,
        }
    }
}
impl<'a> GetUserByEmailBuilder<'a, (&'a str,)> {
    pub const fn build(self) -> GetUserByEmail<'a> {
        let (email,) = self.fields;
        GetUserByEmail { email }
    }
}
pub struct ListUsersRow {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub created_at: std::time::SystemTime,
}
impl ListUsersRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            username: row.try_get(1)?,
            email: row.try_get(2)?,
            full_name: row.try_get(3)?,
            created_at: row.try_get(4)?,
        })
    }
}
pub struct ListUsers {
    limit: i32,
    offset: i32,
}
impl ListUsers {
    pub const QUERY: &'static str = r"SELECT id, username, email, full_name, created_at FROM users
ORDER BY created_at DESC
LIMIT $1
OFFSET $2";
    pub async fn query_many(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Vec<ListUsersRow>, deadpool_postgres::tokio_postgres::Error> {
        let rows = client
            .query(Self::QUERY, &[&self.limit, &self.offset])
            .await?;
        rows.into_iter()
            .map(|r| ListUsersRow::from_row(&r))
            .collect()
    }
}
impl ListUsers {
    pub const fn builder() -> ListUsersBuilder<'static, ((), ())> {
        ListUsersBuilder {
            fields: ((), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct ListUsersBuilder<'a, Fields = ((), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, Offset> ListUsersBuilder<'a, ((), Offset)> {
    pub fn limit(self, limit: i32) -> ListUsersBuilder<'a, (i32, Offset)> {
        let ((), offset) = self.fields;
        let _phantom = self._phantom;
        ListUsersBuilder {
            fields: (limit, offset),
            _phantom,
        }
    }
}
impl<'a, Limit> ListUsersBuilder<'a, (Limit, ())> {
    pub fn offset(self, offset: i32) -> ListUsersBuilder<'a, (Limit, i32)> {
        let (limit, ()) = self.fields;
        let _phantom = self._phantom;
        ListUsersBuilder {
            fields: (limit, offset),
            _phantom,
        }
    }
}
impl<'a> ListUsersBuilder<'a, (i32, i32)> {
    pub const fn build(self) -> ListUsers {
        let (limit, offset) = self.fields;
        ListUsers { limit, offset }
    }
}
pub struct CreateProductRow {
    pub id: uuid::Uuid,
    pub category_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: i32,
    pub stock_quantity: i32,
    pub attributes: Option<serde_json::Value>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}
impl CreateProductRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            category_id: row.try_get(1)?,
            name: row.try_get(2)?,
            description: row.try_get(3)?,
            price: row.try_get(4)?,
            stock_quantity: row.try_get(5)?,
            attributes: row.try_get(6)?,
            created_at: row.try_get(7)?,
            updated_at: row.try_get(8)?,
        })
    }
}
pub struct CreateProduct<'a> {
    category_id: i32,
    name: &'a str,
    description: Option<&'a str>,
    price: i32,
    stock_quantity: i32,
    attributes: Option<&'a serde_json::Value>,
}
impl<'a> CreateProduct<'a> {
    pub const QUERY: &'static str = r"INSERT INTO products (
    category_id, name, description, price, stock_quantity, attributes
) VALUES (
    $1, $2, $3, $4, $5, $6
)
RETURNING id, category_id, name, description, price, stock_quantity, attributes, created_at, updated_at";
    pub async fn query_one(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<CreateProductRow, deadpool_postgres::tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[
                    &self.category_id,
                    &self.name,
                    &self.description,
                    &self.price,
                    &self.stock_quantity,
                    &self.attributes,
                ],
            )
            .await?;
        CreateProductRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Option<CreateProductRow>, deadpool_postgres::tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[
                    &self.category_id,
                    &self.name,
                    &self.description,
                    &self.price,
                    &self.stock_quantity,
                    &self.attributes,
                ],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateProductRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateProduct<'a> {
    pub const fn builder() -> CreateProductBuilder<'a, ((), (), (), (), (), ())> {
        CreateProductBuilder {
            fields: ((), (), (), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct CreateProductBuilder<'a, Fields = ((), (), (), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, Name, Description, Price, StockQuantity, Attributes>
    CreateProductBuilder<'a, ((), Name, Description, Price, StockQuantity, Attributes)>
{
    pub fn category_id(
        self,
        category_id: i32,
    ) -> CreateProductBuilder<'a, (i32, Name, Description, Price, StockQuantity, Attributes)> {
        let ((), name, description, price, stock_quantity, attributes) = self.fields;
        let _phantom = self._phantom;
        CreateProductBuilder {
            fields: (
                category_id,
                name,
                description,
                price,
                stock_quantity,
                attributes,
            ),
            _phantom,
        }
    }
}
impl<'a, CategoryId, Description, Price, StockQuantity, Attributes>
    CreateProductBuilder<
        'a,
        (
            CategoryId,
            (),
            Description,
            Price,
            StockQuantity,
            Attributes,
        ),
    >
{
    pub fn name(
        self,
        name: &'a str,
    ) -> CreateProductBuilder<
        'a,
        (
            CategoryId,
            &'a str,
            Description,
            Price,
            StockQuantity,
            Attributes,
        ),
    > {
        let (category_id, (), description, price, stock_quantity, attributes) = self.fields;
        let _phantom = self._phantom;
        CreateProductBuilder {
            fields: (
                category_id,
                name,
                description,
                price,
                stock_quantity,
                attributes,
            ),
            _phantom,
        }
    }
}
impl<'a, CategoryId, Name, Price, StockQuantity, Attributes>
    CreateProductBuilder<'a, (CategoryId, Name, (), Price, StockQuantity, Attributes)>
{
    pub fn description(
        self,
        description: Option<&'a str>,
    ) -> CreateProductBuilder<
        'a,
        (
            CategoryId,
            Name,
            Option<&'a str>,
            Price,
            StockQuantity,
            Attributes,
        ),
    > {
        let (category_id, name, (), price, stock_quantity, attributes) = self.fields;
        let _phantom = self._phantom;
        CreateProductBuilder {
            fields: (
                category_id,
                name,
                description,
                price,
                stock_quantity,
                attributes,
            ),
            _phantom,
        }
    }
}
impl<'a, CategoryId, Name, Description, StockQuantity, Attributes>
    CreateProductBuilder<'a, (CategoryId, Name, Description, (), StockQuantity, Attributes)>
{
    pub fn price(
        self,
        price: i32,
    ) -> CreateProductBuilder<
        'a,
        (
            CategoryId,
            Name,
            Description,
            i32,
            StockQuantity,
            Attributes,
        ),
    > {
        let (category_id, name, description, (), stock_quantity, attributes) = self.fields;
        let _phantom = self._phantom;
        CreateProductBuilder {
            fields: (
                category_id,
                name,
                description,
                price,
                stock_quantity,
                attributes,
            ),
            _phantom,
        }
    }
}
impl<'a, CategoryId, Name, Description, Price, Attributes>
    CreateProductBuilder<'a, (CategoryId, Name, Description, Price, (), Attributes)>
{
    pub fn stock_quantity(
        self,
        stock_quantity: i32,
    ) -> CreateProductBuilder<'a, (CategoryId, Name, Description, Price, i32, Attributes)> {
        let (category_id, name, description, price, (), attributes) = self.fields;
        let _phantom = self._phantom;
        CreateProductBuilder {
            fields: (
                category_id,
                name,
                description,
                price,
                stock_quantity,
                attributes,
            ),
            _phantom,
        }
    }
}
impl<'a, CategoryId, Name, Description, Price, StockQuantity>
    CreateProductBuilder<'a, (CategoryId, Name, Description, Price, StockQuantity, ())>
{
    pub fn attributes(
        self,
        attributes: Option<&'a serde_json::Value>,
    ) -> CreateProductBuilder<
        'a,
        (
            CategoryId,
            Name,
            Description,
            Price,
            StockQuantity,
            Option<&'a serde_json::Value>,
        ),
    > {
        let (category_id, name, description, price, stock_quantity, ()) = self.fields;
        let _phantom = self._phantom;
        CreateProductBuilder {
            fields: (
                category_id,
                name,
                description,
                price,
                stock_quantity,
                attributes,
            ),
            _phantom,
        }
    }
}
impl<'a>
    CreateProductBuilder<
        'a,
        (
            i32,
            &'a str,
            Option<&'a str>,
            i32,
            i32,
            Option<&'a serde_json::Value>,
        ),
    >
{
    pub const fn build(self) -> CreateProduct<'a> {
        let (category_id, name, description, price, stock_quantity, attributes) = self.fields;
        CreateProduct {
            category_id,
            name,
            description,
            price,
            stock_quantity,
            attributes,
        }
    }
}
pub struct GetProductWithCategoryRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: i32,
    pub stock_quantity: i32,
    pub attributes: Option<serde_json::Value>,
    pub created_at: std::time::SystemTime,
    pub category_name: String,
    pub category_slug: String,
}
impl GetProductWithCategoryRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            name: row.try_get(1)?,
            description: row.try_get(2)?,
            price: row.try_get(3)?,
            stock_quantity: row.try_get(4)?,
            attributes: row.try_get(5)?,
            created_at: row.try_get(6)?,
            category_name: row.try_get(7)?,
            category_slug: row.try_get(8)?,
        })
    }
}
pub struct GetProductWithCategory {
    id: uuid::Uuid,
}
impl GetProductWithCategory {
    pub const QUERY: &'static str = r"SELECT
    p.id,
    p.name,
    p.description,
    p.price,
    p.stock_quantity,
    p.attributes,
    p.created_at,
    c.name as category_name,
    c.slug as category_slug
FROM
    products p
JOIN
    categories c ON p.category_id = c.id
WHERE
    p.id = $1";
    pub async fn query_one(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<GetProductWithCategoryRow, deadpool_postgres::tokio_postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.id]).await?;
        GetProductWithCategoryRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Option<GetProductWithCategoryRow>, deadpool_postgres::tokio_postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.id]).await?;
        match row {
            Some(row) => Ok(Some(GetProductWithCategoryRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl GetProductWithCategory {
    pub const fn builder() -> GetProductWithCategoryBuilder<'static, ((),)> {
        GetProductWithCategoryBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct GetProductWithCategoryBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetProductWithCategoryBuilder<'a, ((),)> {
    pub fn id(self, id: uuid::Uuid) -> GetProductWithCategoryBuilder<'a, (uuid::Uuid,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        GetProductWithCategoryBuilder {
            fields: (id,),
            _phantom,
        }
    }
}
impl<'a> GetProductWithCategoryBuilder<'a, (uuid::Uuid,)> {
    pub const fn build(self) -> GetProductWithCategory {
        let (id,) = self.fields;
        GetProductWithCategory { id }
    }
}
pub struct SearchProductsRow {
    pub id: uuid::Uuid,
    pub category_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: i32,
    pub stock_quantity: i32,
    pub attributes: Option<serde_json::Value>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
    pub average_rating: f64,
}
impl SearchProductsRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            category_id: row.try_get(1)?,
            name: row.try_get(2)?,
            description: row.try_get(3)?,
            price: row.try_get(4)?,
            stock_quantity: row.try_get(5)?,
            attributes: row.try_get(6)?,
            created_at: row.try_get(7)?,
            updated_at: row.try_get(8)?,
            average_rating: row.try_get(9)?,
        })
    }
}
pub struct SearchProducts<'a> {
    limit: i32,
    offset: i32,
    name: Option<&'a str>,
    category_ids: &'a [i32],
    min_price: Option<i32>,
    max_price: Option<i32>,
}
impl<'a> SearchProducts<'a> {
    pub const QUERY: &'static str = r"SELECT
    p.id, p.category_id, p.name, p.description, p.price, p.stock_quantity, p.attributes, p.created_at, p.updated_at,
    (SELECT AVG(r.rating) FROM reviews r WHERE r.product_id = p.id) as average_rating
FROM products p
WHERE
    (p.name ILIKE $3 OR p.description ILIKE $3)
AND
    p.category_id = ANY($4::int[])
AND
    p.price >= $5
AND
    p.price <= $6
AND
    p.stock_quantity > 0
ORDER BY
    p.created_at DESC
LIMIT $1
OFFSET $2";
    pub async fn query_many(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Vec<SearchProductsRow>, deadpool_postgres::tokio_postgres::Error> {
        let rows = client
            .query(
                Self::QUERY,
                &[
                    &self.limit,
                    &self.offset,
                    &self.name,
                    &self.category_ids,
                    &self.min_price,
                    &self.max_price,
                ],
            )
            .await?;
        rows.into_iter()
            .map(|r| SearchProductsRow::from_row(&r))
            .collect()
    }
}
impl<'a> SearchProducts<'a> {
    pub const fn builder() -> SearchProductsBuilder<'a, ((), (), (), (), (), ())> {
        SearchProductsBuilder {
            fields: ((), (), (), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct SearchProductsBuilder<'a, Fields = ((), (), (), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, Offset, Name, CategoryIds, MinPrice, MaxPrice>
    SearchProductsBuilder<'a, ((), Offset, Name, CategoryIds, MinPrice, MaxPrice)>
{
    pub fn limit(
        self,
        limit: i32,
    ) -> SearchProductsBuilder<'a, (i32, Offset, Name, CategoryIds, MinPrice, MaxPrice)> {
        let ((), offset, name, category_ids, min_price, max_price) = self.fields;
        let _phantom = self._phantom;
        SearchProductsBuilder {
            fields: (limit, offset, name, category_ids, min_price, max_price),
            _phantom,
        }
    }
}
impl<'a, Limit, Name, CategoryIds, MinPrice, MaxPrice>
    SearchProductsBuilder<'a, (Limit, (), Name, CategoryIds, MinPrice, MaxPrice)>
{
    pub fn offset(
        self,
        offset: i32,
    ) -> SearchProductsBuilder<'a, (Limit, i32, Name, CategoryIds, MinPrice, MaxPrice)> {
        let (limit, (), name, category_ids, min_price, max_price) = self.fields;
        let _phantom = self._phantom;
        SearchProductsBuilder {
            fields: (limit, offset, name, category_ids, min_price, max_price),
            _phantom,
        }
    }
}
impl<'a, Limit, Offset, CategoryIds, MinPrice, MaxPrice>
    SearchProductsBuilder<'a, (Limit, Offset, (), CategoryIds, MinPrice, MaxPrice)>
{
    pub fn name(
        self,
        name: Option<&'a str>,
    ) -> SearchProductsBuilder<
        'a,
        (
            Limit,
            Offset,
            Option<&'a str>,
            CategoryIds,
            MinPrice,
            MaxPrice,
        ),
    > {
        let (limit, offset, (), category_ids, min_price, max_price) = self.fields;
        let _phantom = self._phantom;
        SearchProductsBuilder {
            fields: (limit, offset, name, category_ids, min_price, max_price),
            _phantom,
        }
    }
}
impl<'a, Limit, Offset, Name, MinPrice, MaxPrice>
    SearchProductsBuilder<'a, (Limit, Offset, Name, (), MinPrice, MaxPrice)>
{
    pub fn category_ids(
        self,
        category_ids: &'a [i32],
    ) -> SearchProductsBuilder<'a, (Limit, Offset, Name, &'a [i32], MinPrice, MaxPrice)> {
        let (limit, offset, name, (), min_price, max_price) = self.fields;
        let _phantom = self._phantom;
        SearchProductsBuilder {
            fields: (limit, offset, name, category_ids, min_price, max_price),
            _phantom,
        }
    }
}
impl<'a, Limit, Offset, Name, CategoryIds, MaxPrice>
    SearchProductsBuilder<'a, (Limit, Offset, Name, CategoryIds, (), MaxPrice)>
{
    pub fn min_price(
        self,
        min_price: Option<i32>,
    ) -> SearchProductsBuilder<'a, (Limit, Offset, Name, CategoryIds, Option<i32>, MaxPrice)> {
        let (limit, offset, name, category_ids, (), max_price) = self.fields;
        let _phantom = self._phantom;
        SearchProductsBuilder {
            fields: (limit, offset, name, category_ids, min_price, max_price),
            _phantom,
        }
    }
}
impl<'a, Limit, Offset, Name, CategoryIds, MinPrice>
    SearchProductsBuilder<'a, (Limit, Offset, Name, CategoryIds, MinPrice, ())>
{
    pub fn max_price(
        self,
        max_price: Option<i32>,
    ) -> SearchProductsBuilder<'a, (Limit, Offset, Name, CategoryIds, MinPrice, Option<i32>)> {
        let (limit, offset, name, category_ids, min_price, ()) = self.fields;
        let _phantom = self._phantom;
        SearchProductsBuilder {
            fields: (limit, offset, name, category_ids, min_price, max_price),
            _phantom,
        }
    }
}
impl<'a>
    SearchProductsBuilder<
        'a,
        (
            i32,
            i32,
            Option<&'a str>,
            &'a [i32],
            Option<i32>,
            Option<i32>,
        ),
    >
{
    pub const fn build(self) -> SearchProducts<'a> {
        let (limit, offset, name, category_ids, min_price, max_price) = self.fields;
        SearchProducts {
            limit,
            offset,
            name,
            category_ids,
            min_price,
            max_price,
        }
    }
}
pub struct GetProductsWithSpecificAttributeRow {
    pub id: uuid::Uuid,
    pub category_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: i32,
    pub stock_quantity: i32,
    pub attributes: Option<serde_json::Value>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}
impl GetProductsWithSpecificAttributeRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            category_id: row.try_get(1)?,
            name: row.try_get(2)?,
            description: row.try_get(3)?,
            price: row.try_get(4)?,
            stock_quantity: row.try_get(5)?,
            attributes: row.try_get(6)?,
            created_at: row.try_get(7)?,
            updated_at: row.try_get(8)?,
        })
    }
}
pub struct GetProductsWithSpecificAttribute<'a> {
    column_1: &'a serde_json::Value,
}
impl<'a> GetProductsWithSpecificAttribute<'a> {
    pub const QUERY: &'static str = r"SELECT id, category_id, name, description, price, stock_quantity, attributes, created_at, updated_at FROM products
WHERE attributes @> $1::jsonb";
    pub async fn query_many(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Vec<GetProductsWithSpecificAttributeRow>, deadpool_postgres::tokio_postgres::Error>
    {
        let rows = client.query(Self::QUERY, &[&self.column_1]).await?;
        rows.into_iter()
            .map(|r| GetProductsWithSpecificAttributeRow::from_row(&r))
            .collect()
    }
}
impl<'a> GetProductsWithSpecificAttribute<'a> {
    pub const fn builder() -> GetProductsWithSpecificAttributeBuilder<'a, ((),)> {
        GetProductsWithSpecificAttributeBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct GetProductsWithSpecificAttributeBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetProductsWithSpecificAttributeBuilder<'a, ((),)> {
    pub fn column_1(
        self,
        column_1: &'a serde_json::Value,
    ) -> GetProductsWithSpecificAttributeBuilder<'a, (&'a serde_json::Value,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        GetProductsWithSpecificAttributeBuilder {
            fields: (column_1,),
            _phantom,
        }
    }
}
impl<'a> GetProductsWithSpecificAttributeBuilder<'a, (&'a serde_json::Value,)> {
    pub const fn build(self) -> GetProductsWithSpecificAttribute<'a> {
        let (column_1,) = self.fields;
        GetProductsWithSpecificAttribute { column_1 }
    }
}
pub struct UpdateProductStockRow {}
impl UpdateProductStockRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {})
    }
}
pub struct UpdateProductStock {
    id: uuid::Uuid,
    add_quantity: i32,
}
impl UpdateProductStock {
    pub const QUERY: &'static str = r"UPDATE products
SET stock_quantity = stock_quantity + $2
WHERE id = $1";
    pub async fn execute(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<u64, deadpool_postgres::tokio_postgres::Error> {
        client
            .execute(Self::QUERY, &[&self.id, &self.add_quantity])
            .await
    }
}
impl UpdateProductStock {
    pub const fn builder() -> UpdateProductStockBuilder<'static, ((), ())> {
        UpdateProductStockBuilder {
            fields: ((), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct UpdateProductStockBuilder<'a, Fields = ((), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, AddQuantity> UpdateProductStockBuilder<'a, ((), AddQuantity)> {
    pub fn id(self, id: uuid::Uuid) -> UpdateProductStockBuilder<'a, (uuid::Uuid, AddQuantity)> {
        let ((), add_quantity) = self.fields;
        let _phantom = self._phantom;
        UpdateProductStockBuilder {
            fields: (id, add_quantity),
            _phantom,
        }
    }
}
impl<'a, Id> UpdateProductStockBuilder<'a, (Id, ())> {
    pub fn add_quantity(self, add_quantity: i32) -> UpdateProductStockBuilder<'a, (Id, i32)> {
        let (id, ()) = self.fields;
        let _phantom = self._phantom;
        UpdateProductStockBuilder {
            fields: (id, add_quantity),
            _phantom,
        }
    }
}
impl<'a> UpdateProductStockBuilder<'a, (uuid::Uuid, i32)> {
    pub const fn build(self) -> UpdateProductStock {
        let (id, add_quantity) = self.fields;
        UpdateProductStock { id, add_quantity }
    }
}
pub struct CreateOrderRow {
    pub id: i64,
    pub user_id: uuid::Uuid,
    pub status: OrderStatus,
    pub total_amount: i32,
    pub ordered_at: std::time::SystemTime,
}
impl CreateOrderRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            user_id: row.try_get(1)?,
            status: row.try_get(2)?,
            total_amount: row.try_get(3)?,
            ordered_at: row.try_get(4)?,
        })
    }
}
pub struct CreateOrder {
    user_id: uuid::Uuid,
    status: OrderStatus,
    total_amount: i32,
}
impl CreateOrder {
    pub const QUERY: &'static str = r"INSERT INTO orders (user_id, status, total_amount)
VALUES ($1, $2, $3)
RETURNING id, user_id, status, total_amount, ordered_at";
    pub async fn query_one(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<CreateOrderRow, deadpool_postgres::tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[&self.user_id, &self.status, &self.total_amount],
            )
            .await?;
        CreateOrderRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Option<CreateOrderRow>, deadpool_postgres::tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[&self.user_id, &self.status, &self.total_amount],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateOrderRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl CreateOrder {
    pub const fn builder() -> CreateOrderBuilder<'static, ((), (), ())> {
        CreateOrderBuilder {
            fields: ((), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct CreateOrderBuilder<'a, Fields = ((), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, Status, TotalAmount> CreateOrderBuilder<'a, ((), Status, TotalAmount)> {
    pub fn user_id(
        self,
        user_id: uuid::Uuid,
    ) -> CreateOrderBuilder<'a, (uuid::Uuid, Status, TotalAmount)> {
        let ((), status, total_amount) = self.fields;
        let _phantom = self._phantom;
        CreateOrderBuilder {
            fields: (user_id, status, total_amount),
            _phantom,
        }
    }
}
impl<'a, UserId, TotalAmount> CreateOrderBuilder<'a, (UserId, (), TotalAmount)> {
    pub fn status(
        self,
        status: OrderStatus,
    ) -> CreateOrderBuilder<'a, (UserId, OrderStatus, TotalAmount)> {
        let (user_id, (), total_amount) = self.fields;
        let _phantom = self._phantom;
        CreateOrderBuilder {
            fields: (user_id, status, total_amount),
            _phantom,
        }
    }
}
impl<'a, UserId, Status> CreateOrderBuilder<'a, (UserId, Status, ())> {
    pub fn total_amount(self, total_amount: i32) -> CreateOrderBuilder<'a, (UserId, Status, i32)> {
        let (user_id, status, ()) = self.fields;
        let _phantom = self._phantom;
        CreateOrderBuilder {
            fields: (user_id, status, total_amount),
            _phantom,
        }
    }
}
impl<'a> CreateOrderBuilder<'a, (uuid::Uuid, OrderStatus, i32)> {
    pub const fn build(self) -> CreateOrder {
        let (user_id, status, total_amount) = self.fields;
        CreateOrder {
            user_id,
            status,
            total_amount,
        }
    }
}
pub struct CreateOrderItemRow {
    pub id: i64,
    pub order_id: i64,
    pub product_id: uuid::Uuid,
    pub quantity: i32,
    pub price_at_purchase: i32,
}
impl CreateOrderItemRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            order_id: row.try_get(1)?,
            product_id: row.try_get(2)?,
            quantity: row.try_get(3)?,
            price_at_purchase: row.try_get(4)?,
        })
    }
}
pub struct CreateOrderItem {
    order_id: i64,
    product_id: uuid::Uuid,
    quantity: i32,
    price_at_purchase: i32,
}
impl CreateOrderItem {
    pub const QUERY: &'static str = r"INSERT INTO order_items (order_id, product_id, quantity, price_at_purchase)
VALUES ($1, $2, $3, $4)
RETURNING id, order_id, product_id, quantity, price_at_purchase";
    pub async fn query_one(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<CreateOrderItemRow, deadpool_postgres::tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[
                    &self.order_id,
                    &self.product_id,
                    &self.quantity,
                    &self.price_at_purchase,
                ],
            )
            .await?;
        CreateOrderItemRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Option<CreateOrderItemRow>, deadpool_postgres::tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[
                    &self.order_id,
                    &self.product_id,
                    &self.quantity,
                    &self.price_at_purchase,
                ],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateOrderItemRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl CreateOrderItem {
    pub const fn builder() -> CreateOrderItemBuilder<'static, ((), (), (), ())> {
        CreateOrderItemBuilder {
            fields: ((), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct CreateOrderItemBuilder<'a, Fields = ((), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, ProductId, Quantity, PriceAtPurchase>
    CreateOrderItemBuilder<'a, ((), ProductId, Quantity, PriceAtPurchase)>
{
    pub fn order_id(
        self,
        order_id: i64,
    ) -> CreateOrderItemBuilder<'a, (i64, ProductId, Quantity, PriceAtPurchase)> {
        let ((), product_id, quantity, price_at_purchase) = self.fields;
        let _phantom = self._phantom;
        CreateOrderItemBuilder {
            fields: (order_id, product_id, quantity, price_at_purchase),
            _phantom,
        }
    }
}
impl<'a, OrderId, Quantity, PriceAtPurchase>
    CreateOrderItemBuilder<'a, (OrderId, (), Quantity, PriceAtPurchase)>
{
    pub fn product_id(
        self,
        product_id: uuid::Uuid,
    ) -> CreateOrderItemBuilder<'a, (OrderId, uuid::Uuid, Quantity, PriceAtPurchase)> {
        let (order_id, (), quantity, price_at_purchase) = self.fields;
        let _phantom = self._phantom;
        CreateOrderItemBuilder {
            fields: (order_id, product_id, quantity, price_at_purchase),
            _phantom,
        }
    }
}
impl<'a, OrderId, ProductId, PriceAtPurchase>
    CreateOrderItemBuilder<'a, (OrderId, ProductId, (), PriceAtPurchase)>
{
    pub fn quantity(
        self,
        quantity: i32,
    ) -> CreateOrderItemBuilder<'a, (OrderId, ProductId, i32, PriceAtPurchase)> {
        let (order_id, product_id, (), price_at_purchase) = self.fields;
        let _phantom = self._phantom;
        CreateOrderItemBuilder {
            fields: (order_id, product_id, quantity, price_at_purchase),
            _phantom,
        }
    }
}
impl<'a, OrderId, ProductId, Quantity>
    CreateOrderItemBuilder<'a, (OrderId, ProductId, Quantity, ())>
{
    pub fn price_at_purchase(
        self,
        price_at_purchase: i32,
    ) -> CreateOrderItemBuilder<'a, (OrderId, ProductId, Quantity, i32)> {
        let (order_id, product_id, quantity, ()) = self.fields;
        let _phantom = self._phantom;
        CreateOrderItemBuilder {
            fields: (order_id, product_id, quantity, price_at_purchase),
            _phantom,
        }
    }
}
impl<'a> CreateOrderItemBuilder<'a, (i64, uuid::Uuid, i32, i32)> {
    pub const fn build(self) -> CreateOrderItem {
        let (order_id, product_id, quantity, price_at_purchase) = self.fields;
        CreateOrderItem {
            order_id,
            product_id,
            quantity,
            price_at_purchase,
        }
    }
}
pub struct GetOrderDetailsRow {
    pub order_id: i64,
    pub status: OrderStatus,
    pub total_amount: i32,
    pub ordered_at: std::time::SystemTime,
    pub user_id: uuid::Uuid,
    pub username: String,
    pub email: String,
}
impl GetOrderDetailsRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            order_id: row.try_get(0)?,
            status: row.try_get(1)?,
            total_amount: row.try_get(2)?,
            ordered_at: row.try_get(3)?,
            user_id: row.try_get(4)?,
            username: row.try_get(5)?,
            email: row.try_get(6)?,
        })
    }
}
pub struct GetOrderDetails {
    id: i64,
}
impl GetOrderDetails {
    pub const QUERY: &'static str = r"SELECT
    o.id as order_id,
    o.status,
    o.total_amount,
    o.ordered_at,
    u.id as user_id,
    u.username,
    u.email
FROM orders o
JOIN users u ON o.user_id = u.id
WHERE o.id = $1";
    pub async fn query_one(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<GetOrderDetailsRow, deadpool_postgres::tokio_postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.id]).await?;
        GetOrderDetailsRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Option<GetOrderDetailsRow>, deadpool_postgres::tokio_postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.id]).await?;
        match row {
            Some(row) => Ok(Some(GetOrderDetailsRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl GetOrderDetails {
    pub const fn builder() -> GetOrderDetailsBuilder<'static, ((),)> {
        GetOrderDetailsBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct GetOrderDetailsBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetOrderDetailsBuilder<'a, ((),)> {
    pub fn id(self, id: i64) -> GetOrderDetailsBuilder<'a, (i64,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        GetOrderDetailsBuilder {
            fields: (id,),
            _phantom,
        }
    }
}
impl<'a> GetOrderDetailsBuilder<'a, (i64,)> {
    pub const fn build(self) -> GetOrderDetails {
        let (id,) = self.fields;
        GetOrderDetails { id }
    }
}
pub struct ListOrderItemsByOrderIdRow {
    pub quantity: i32,
    pub price_at_purchase: i32,
    pub product_id: uuid::Uuid,
    pub product_name: String,
}
impl ListOrderItemsByOrderIdRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            quantity: row.try_get(0)?,
            price_at_purchase: row.try_get(1)?,
            product_id: row.try_get(2)?,
            product_name: row.try_get(3)?,
        })
    }
}
pub struct ListOrderItemsByOrderId {
    order_id: i64,
}
impl ListOrderItemsByOrderId {
    pub const QUERY: &'static str = r"SELECT
    oi.quantity,
    oi.price_at_purchase,
    p.id as product_id,
    p.name as product_name
FROM order_items oi
JOIN products p ON oi.product_id = p.id
WHERE oi.order_id = $1";
    pub async fn query_many(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Vec<ListOrderItemsByOrderIdRow>, deadpool_postgres::tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[&self.order_id]).await?;
        rows.into_iter()
            .map(|r| ListOrderItemsByOrderIdRow::from_row(&r))
            .collect()
    }
}
impl ListOrderItemsByOrderId {
    pub const fn builder() -> ListOrderItemsByOrderIdBuilder<'static, ((),)> {
        ListOrderItemsByOrderIdBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct ListOrderItemsByOrderIdBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> ListOrderItemsByOrderIdBuilder<'a, ((),)> {
    pub fn order_id(self, order_id: i64) -> ListOrderItemsByOrderIdBuilder<'a, (i64,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        ListOrderItemsByOrderIdBuilder {
            fields: (order_id,),
            _phantom,
        }
    }
}
impl<'a> ListOrderItemsByOrderIdBuilder<'a, (i64,)> {
    pub const fn build(self) -> ListOrderItemsByOrderId {
        let (order_id,) = self.fields;
        ListOrderItemsByOrderId { order_id }
    }
}
pub struct CreateReviewRow {
    pub id: i64,
    pub user_id: uuid::Uuid,
    pub product_id: uuid::Uuid,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: std::time::SystemTime,
}
impl CreateReviewRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            user_id: row.try_get(1)?,
            product_id: row.try_get(2)?,
            rating: row.try_get(3)?,
            comment: row.try_get(4)?,
            created_at: row.try_get(5)?,
        })
    }
}
pub struct CreateReview<'a> {
    user_id: uuid::Uuid,
    product_id: uuid::Uuid,
    rating: i32,
    comment: Option<&'a str>,
}
impl<'a> CreateReview<'a> {
    pub const QUERY: &'static str = r"INSERT INTO reviews (user_id, product_id, rating, comment)
VALUES ($1, $2, $3, $4)
RETURNING id, user_id, product_id, rating, comment, created_at";
    pub async fn query_one(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<CreateReviewRow, deadpool_postgres::tokio_postgres::Error> {
        let row = client
            .query_one(
                Self::QUERY,
                &[&self.user_id, &self.product_id, &self.rating, &self.comment],
            )
            .await?;
        CreateReviewRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Option<CreateReviewRow>, deadpool_postgres::tokio_postgres::Error> {
        let row = client
            .query_opt(
                Self::QUERY,
                &[&self.user_id, &self.product_id, &self.rating, &self.comment],
            )
            .await?;
        match row {
            Some(row) => Ok(Some(CreateReviewRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateReview<'a> {
    pub const fn builder() -> CreateReviewBuilder<'a, ((), (), (), ())> {
        CreateReviewBuilder {
            fields: ((), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct CreateReviewBuilder<'a, Fields = ((), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, ProductId, Rating, Comment> CreateReviewBuilder<'a, ((), ProductId, Rating, Comment)> {
    pub fn user_id(
        self,
        user_id: uuid::Uuid,
    ) -> CreateReviewBuilder<'a, (uuid::Uuid, ProductId, Rating, Comment)> {
        let ((), product_id, rating, comment) = self.fields;
        let _phantom = self._phantom;
        CreateReviewBuilder {
            fields: (user_id, product_id, rating, comment),
            _phantom,
        }
    }
}
impl<'a, UserId, Rating, Comment> CreateReviewBuilder<'a, (UserId, (), Rating, Comment)> {
    pub fn product_id(
        self,
        product_id: uuid::Uuid,
    ) -> CreateReviewBuilder<'a, (UserId, uuid::Uuid, Rating, Comment)> {
        let (user_id, (), rating, comment) = self.fields;
        let _phantom = self._phantom;
        CreateReviewBuilder {
            fields: (user_id, product_id, rating, comment),
            _phantom,
        }
    }
}
impl<'a, UserId, ProductId, Comment> CreateReviewBuilder<'a, (UserId, ProductId, (), Comment)> {
    pub fn rating(self, rating: i32) -> CreateReviewBuilder<'a, (UserId, ProductId, i32, Comment)> {
        let (user_id, product_id, (), comment) = self.fields;
        let _phantom = self._phantom;
        CreateReviewBuilder {
            fields: (user_id, product_id, rating, comment),
            _phantom,
        }
    }
}
impl<'a, UserId, ProductId, Rating> CreateReviewBuilder<'a, (UserId, ProductId, Rating, ())> {
    pub fn comment(
        self,
        comment: Option<&'a str>,
    ) -> CreateReviewBuilder<'a, (UserId, ProductId, Rating, Option<&'a str>)> {
        let (user_id, product_id, rating, ()) = self.fields;
        let _phantom = self._phantom;
        CreateReviewBuilder {
            fields: (user_id, product_id, rating, comment),
            _phantom,
        }
    }
}
impl<'a> CreateReviewBuilder<'a, (uuid::Uuid, uuid::Uuid, i32, Option<&'a str>)> {
    pub const fn build(self) -> CreateReview<'a> {
        let (user_id, product_id, rating, comment) = self.fields;
        CreateReview {
            user_id,
            product_id,
            rating,
            comment,
        }
    }
}
pub struct GetProductAverageRatingRow {
    pub product_id: uuid::Uuid,
    pub average_rating: f64,
    pub review_count: i64,
}
impl GetProductAverageRatingRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            product_id: row.try_get(0)?,
            average_rating: row.try_get(1)?,
            review_count: row.try_get(2)?,
        })
    }
}
pub struct GetProductAverageRating {
    product_id: uuid::Uuid,
}
impl GetProductAverageRating {
    pub const QUERY: &'static str = r"SELECT
    product_id,
    AVG(rating)::float as average_rating,
    COUNT(id) as review_count
FROM reviews
WHERE product_id = $1
GROUP BY product_id";
    pub async fn query_one(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<GetProductAverageRatingRow, deadpool_postgres::tokio_postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.product_id]).await?;
        GetProductAverageRatingRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Option<GetProductAverageRatingRow>, deadpool_postgres::tokio_postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.product_id]).await?;
        match row {
            Some(row) => Ok(Some(GetProductAverageRatingRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl GetProductAverageRating {
    pub const fn builder() -> GetProductAverageRatingBuilder<'static, ((),)> {
        GetProductAverageRatingBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct GetProductAverageRatingBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetProductAverageRatingBuilder<'a, ((),)> {
    pub fn product_id(
        self,
        product_id: uuid::Uuid,
    ) -> GetProductAverageRatingBuilder<'a, (uuid::Uuid,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        GetProductAverageRatingBuilder {
            fields: (product_id,),
            _phantom,
        }
    }
}
impl<'a> GetProductAverageRatingBuilder<'a, (uuid::Uuid,)> {
    pub const fn build(self) -> GetProductAverageRating {
        let (product_id,) = self.fields;
        GetProductAverageRating { product_id }
    }
}
pub struct GetCategorySalesRankingRow {
    pub category_id: i32,
    pub category_name: String,
    pub total_sales: i64,
    pub total_orders: i64,
}
impl GetCategorySalesRankingRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            category_id: row.try_get(0)?,
            category_name: row.try_get(1)?,
            total_sales: row.try_get(2)?,
            total_orders: row.try_get(3)?,
        })
    }
}
pub struct GetCategorySalesRanking;
impl GetCategorySalesRanking {
    pub const QUERY: &'static str = r"SELECT
    c.id as category_id,
    c.name as category_name,
    SUM(oi.quantity * oi.price_at_purchase) as total_sales,
    COUNT(DISTINCT o.id) as total_orders
FROM categories c
JOIN products p ON c.id = p.category_id
JOIN order_items oi ON p.id = oi.product_id
JOIN orders o ON oi.order_id = o.id
WHERE o.status IN ('delivered', 'shipped')
GROUP BY c.id, c.name
ORDER BY total_sales DESC";
    pub async fn query_many(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Vec<GetCategorySalesRankingRow>, deadpool_postgres::tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[]).await?;
        rows.into_iter()
            .map(|r| GetCategorySalesRankingRow::from_row(&r))
            .collect()
    }
}
pub struct DeleteUserAndRelatedDataRow {}
impl DeleteUserAndRelatedDataRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {})
    }
}
pub struct DeleteUserAndRelatedData {
    id: uuid::Uuid,
}
impl DeleteUserAndRelatedData {
    pub const QUERY: &'static str = r"DELETE FROM users WHERE id = $1";
    pub async fn execute(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<u64, deadpool_postgres::tokio_postgres::Error> {
        client.execute(Self::QUERY, &[&self.id]).await
    }
}
impl DeleteUserAndRelatedData {
    pub const fn builder() -> DeleteUserAndRelatedDataBuilder<'static, ((),)> {
        DeleteUserAndRelatedDataBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct DeleteUserAndRelatedDataBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> DeleteUserAndRelatedDataBuilder<'a, ((),)> {
    pub fn id(self, id: uuid::Uuid) -> DeleteUserAndRelatedDataBuilder<'a, (uuid::Uuid,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        DeleteUserAndRelatedDataBuilder {
            fields: (id,),
            _phantom,
        }
    }
}
impl<'a> DeleteUserAndRelatedDataBuilder<'a, (uuid::Uuid,)> {
    pub const fn build(self) -> DeleteUserAndRelatedData {
        let (id,) = self.fields;
        DeleteUserAndRelatedData { id }
    }
}
