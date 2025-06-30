pub struct GetAuthorRow {
    pub authors_id: i64,
    pub authors_name: String,
    pub authors_bio: Option<String>,
}
impl GetAuthorRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            authors_id: row.try_get(0)?,
            authors_name: row.try_get(1)?,
            authors_bio: row.try_get(2)?,
        })
    }
}
pub struct GetAuthor {
    authors_id: i64,
}
impl GetAuthor {
    pub const QUERY: &'static str = r"SELECT id, name, bio FROM authors
WHERE id = $1 LIMIT 1";
    pub async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<GetAuthorRow, tokio_postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.authors_id]).await?;
        GetAuthorRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<GetAuthorRow>, tokio_postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.authors_id]).await?;
        match row {
            Some(row) => Ok(Some(GetAuthorRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl GetAuthor {
    pub const fn builder() -> GetAuthorBuilder<'static, ((),)> {
        GetAuthorBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct GetAuthorBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetAuthorBuilder<'a, ((),)> {
    pub fn authors_id(self, authors_id: i64) -> GetAuthorBuilder<'a, (i64,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        GetAuthorBuilder {
            fields: (authors_id,),
            _phantom,
        }
    }
}
impl<'a> GetAuthorBuilder<'a, (i64,)> {
    pub const fn build(self) -> GetAuthor {
        let (authors_id,) = self.fields;
        GetAuthor { authors_id }
    }
}
pub struct ListAuthorsRow {
    pub authors_id: i64,
    pub authors_name: String,
    pub authors_bio: Option<String>,
}
impl ListAuthorsRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            authors_id: row.try_get(0)?,
            authors_name: row.try_get(1)?,
            authors_bio: row.try_get(2)?,
        })
    }
}
pub struct ListAuthors;
impl ListAuthors {
    pub const QUERY: &'static str = r"SELECT id, name, bio FROM authors
ORDER BY name";
    pub async fn query_many(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Vec<ListAuthorsRow>, tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[]).await?;
        rows.into_iter()
            .map(|r| ListAuthorsRow::from_row(&r))
            .collect()
    }
}
pub struct CreateAuthorRow {
    pub authors_id: i64,
    pub authors_name: String,
    pub authors_bio: Option<String>,
}
impl CreateAuthorRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            authors_id: row.try_get(0)?,
            authors_name: row.try_get(1)?,
            authors_bio: row.try_get(2)?,
        })
    }
}
pub struct CreateAuthor<'a> {
    authors_name: &'a str,
    authors_bio: Option<&'a str>,
}
impl<'a> CreateAuthor<'a> {
    pub const QUERY: &'static str = r"INSERT INTO authors (
          name, bio
) VALUES (
  $1, $2
)
RETURNING id, name, bio";
    pub async fn query_one(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<CreateAuthorRow, tokio_postgres::Error> {
        let row = client
            .query_one(Self::QUERY, &[&self.authors_name, &self.authors_bio])
            .await?;
        CreateAuthorRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<Option<CreateAuthorRow>, tokio_postgres::Error> {
        let row = client
            .query_opt(Self::QUERY, &[&self.authors_name, &self.authors_bio])
            .await?;
        match row {
            Some(row) => Ok(Some(CreateAuthorRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateAuthor<'a> {
    pub const fn builder() -> CreateAuthorBuilder<'a, ((), ())> {
        CreateAuthorBuilder {
            fields: ((), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct CreateAuthorBuilder<'a, Fields = ((), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, AuthorsBio> CreateAuthorBuilder<'a, ((), AuthorsBio)> {
    pub fn authors_name(
        self,
        authors_name: &'a str,
    ) -> CreateAuthorBuilder<'a, (&'a str, AuthorsBio)> {
        let ((), authors_bio) = self.fields;
        let _phantom = self._phantom;
        CreateAuthorBuilder {
            fields: (authors_name, authors_bio),
            _phantom,
        }
    }
}
impl<'a, AuthorsName> CreateAuthorBuilder<'a, (AuthorsName, ())> {
    pub fn authors_bio(
        self,
        authors_bio: Option<&'a str>,
    ) -> CreateAuthorBuilder<'a, (AuthorsName, Option<&'a str>)> {
        let (authors_name, ()) = self.fields;
        let _phantom = self._phantom;
        CreateAuthorBuilder {
            fields: (authors_name, authors_bio),
            _phantom,
        }
    }
}
impl<'a> CreateAuthorBuilder<'a, (&'a str, Option<&'a str>)> {
    pub const fn build(self) -> CreateAuthor<'a> {
        let (authors_name, authors_bio) = self.fields;
        CreateAuthor {
            authors_name,
            authors_bio,
        }
    }
}
pub struct DeleteAuthorRow {}
impl DeleteAuthorRow {
    fn from_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {})
    }
}
pub struct DeleteAuthor {
    authors_id: i64,
}
impl DeleteAuthor {
    pub const QUERY: &'static str = r"DELETE FROM authors
WHERE id = $1";
    pub async fn execute(
        &self,
        client: &impl tokio_postgres::GenericClient,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(Self::QUERY, &[&self.authors_id]).await
    }
}
impl DeleteAuthor {
    pub const fn builder() -> DeleteAuthorBuilder<'static, ((),)> {
        DeleteAuthorBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct DeleteAuthorBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> DeleteAuthorBuilder<'a, ((),)> {
    pub fn authors_id(self, authors_id: i64) -> DeleteAuthorBuilder<'a, (i64,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        DeleteAuthorBuilder {
            fields: (authors_id,),
            _phantom,
        }
    }
}
impl<'a> DeleteAuthorBuilder<'a, (i64,)> {
    pub const fn build(self) -> DeleteAuthor {
        let (authors_id,) = self.fields;
        DeleteAuthor { authors_id }
    }
}
