#[derive(Debug, Clone, Copy, postgres_types::ToSql, postgres_types::FromSql)]
#[postgres(name = "book_type")]
pub enum BookType {
    #[postgres(name = "FICTION")]
    Fiction,
    #[postgres(name = "NONFICTION")]
    Nonfiction,
}
pub struct GetAuthorRow {
    pub authors_author_id: i32,
    pub authors_name: String,
}
impl GetAuthorRow {
    fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {
            authors_author_id: row.try_get(0)?,
            authors_name: row.try_get(1)?,
        })
    }
}
pub struct GetAuthor {
    authors_author_id: i32,
}
impl GetAuthor {
    pub const QUERY: &'static str = r"SELECT author_id, name FROM authors
WHERE author_id = $1";
    pub fn query_one(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<GetAuthorRow, postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.authors_author_id])?;
        GetAuthorRow::from_row(&row)
    }
    pub fn query_opt(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<Option<GetAuthorRow>, postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.authors_author_id])?;
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
    pub fn authors_author_id(self, authors_author_id: i32) -> GetAuthorBuilder<'a, (i32,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        GetAuthorBuilder {
            fields: (authors_author_id,),
            _phantom,
        }
    }
}
impl<'a> GetAuthorBuilder<'a, (i32,)> {
    pub const fn build(self) -> GetAuthor {
        let (authors_author_id,) = self.fields;
        GetAuthor { authors_author_id }
    }
}
pub struct GetBookRow {
    pub books_book_id: i32,
    pub books_author_id: i32,
    pub books_isbn: String,
    pub books_book_type: BookType,
    pub books_title: String,
    pub books_year: i32,
    pub books_available: std::time::SystemTime,
    pub books_tags: Vec<String>,
}
impl GetBookRow {
    fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {
            books_book_id: row.try_get(0)?,
            books_author_id: row.try_get(1)?,
            books_isbn: row.try_get(2)?,
            books_book_type: row.try_get(3)?,
            books_title: row.try_get(4)?,
            books_year: row.try_get(5)?,
            books_available: row.try_get(6)?,
            books_tags: row.try_get(7)?,
        })
    }
}
pub struct GetBook {
    books_book_id: i32,
}
impl GetBook {
    pub const QUERY: &'static str = r"SELECT book_id, author_id, isbn, book_type, title, year, available, tags FROM books
WHERE book_id = $1";
    pub fn query_one(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<GetBookRow, postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.books_book_id])?;
        GetBookRow::from_row(&row)
    }
    pub fn query_opt(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<Option<GetBookRow>, postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.books_book_id])?;
        match row {
            Some(row) => Ok(Some(GetBookRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl GetBook {
    pub const fn builder() -> GetBookBuilder<'static, ((),)> {
        GetBookBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct GetBookBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> GetBookBuilder<'a, ((),)> {
    pub fn books_book_id(self, books_book_id: i32) -> GetBookBuilder<'a, (i32,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        GetBookBuilder {
            fields: (books_book_id,),
            _phantom,
        }
    }
}
impl<'a> GetBookBuilder<'a, (i32,)> {
    pub const fn build(self) -> GetBook {
        let (books_book_id,) = self.fields;
        GetBook { books_book_id }
    }
}
pub struct DeleteBookRow {}
impl DeleteBookRow {
    fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {})
    }
}
pub struct DeleteBook {
    books_book_id: i32,
}
impl DeleteBook {
    pub const QUERY: &'static str = r"DELETE FROM books
WHERE book_id = $1";
    pub fn execute(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<u64, postgres::Error> {
        client.execute(Self::QUERY, &[&self.books_book_id])
    }
}
impl DeleteBook {
    pub const fn builder() -> DeleteBookBuilder<'static, ((),)> {
        DeleteBookBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct DeleteBookBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> DeleteBookBuilder<'a, ((),)> {
    pub fn books_book_id(self, books_book_id: i32) -> DeleteBookBuilder<'a, (i32,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        DeleteBookBuilder {
            fields: (books_book_id,),
            _phantom,
        }
    }
}
impl<'a> DeleteBookBuilder<'a, (i32,)> {
    pub const fn build(self) -> DeleteBook {
        let (books_book_id,) = self.fields;
        DeleteBook { books_book_id }
    }
}
pub struct BooksByTitleYearRow {
    pub books_book_id: i32,
    pub books_author_id: i32,
    pub books_isbn: String,
    pub books_book_type: BookType,
    pub books_title: String,
    pub books_year: i32,
    pub books_available: std::time::SystemTime,
    pub books_tags: Vec<String>,
}
impl BooksByTitleYearRow {
    fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {
            books_book_id: row.try_get(0)?,
            books_author_id: row.try_get(1)?,
            books_isbn: row.try_get(2)?,
            books_book_type: row.try_get(3)?,
            books_title: row.try_get(4)?,
            books_year: row.try_get(5)?,
            books_available: row.try_get(6)?,
            books_tags: row.try_get(7)?,
        })
    }
}
pub struct BooksByTitleYear<'a> {
    books_title: &'a str,
    books_year: i32,
}
impl<'a> BooksByTitleYear<'a> {
    pub const QUERY: &'static str = r"SELECT book_id, author_id, isbn, book_type, title, year, available, tags FROM books
WHERE title = $1 AND year = $2";
    pub fn query_many(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<Vec<BooksByTitleYearRow>, postgres::Error> {
        let rows = client.query(Self::QUERY, &[&self.books_title, &self.books_year])?;
        rows.into_iter()
            .map(|r| BooksByTitleYearRow::from_row(&r))
            .collect()
    }
}
impl<'a> BooksByTitleYear<'a> {
    pub const fn builder() -> BooksByTitleYearBuilder<'a, ((), ())> {
        BooksByTitleYearBuilder {
            fields: ((), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct BooksByTitleYearBuilder<'a, Fields = ((), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, BooksYear> BooksByTitleYearBuilder<'a, ((), BooksYear)> {
    pub fn books_title(
        self,
        books_title: &'a str,
    ) -> BooksByTitleYearBuilder<'a, (&'a str, BooksYear)> {
        let ((), books_year) = self.fields;
        let _phantom = self._phantom;
        BooksByTitleYearBuilder {
            fields: (books_title, books_year),
            _phantom,
        }
    }
}
impl<'a, BooksTitle> BooksByTitleYearBuilder<'a, (BooksTitle, ())> {
    pub fn books_year(self, books_year: i32) -> BooksByTitleYearBuilder<'a, (BooksTitle, i32)> {
        let (books_title, ()) = self.fields;
        let _phantom = self._phantom;
        BooksByTitleYearBuilder {
            fields: (books_title, books_year),
            _phantom,
        }
    }
}
impl<'a> BooksByTitleYearBuilder<'a, (&'a str, i32)> {
    pub const fn build(self) -> BooksByTitleYear<'a> {
        let (books_title, books_year) = self.fields;
        BooksByTitleYear {
            books_title,
            books_year,
        }
    }
}
pub struct BooksByTagsRow {
    pub books_book_id: i32,
    pub books_title: String,
    pub authors_name: Option<String>,
    pub books_isbn: String,
    pub books_tags: Vec<String>,
}
impl BooksByTagsRow {
    fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {
            books_book_id: row.try_get(0)?,
            books_title: row.try_get(1)?,
            authors_name: row.try_get(2)?,
            books_isbn: row.try_get(3)?,
            books_tags: row.try_get(4)?,
        })
    }
}
pub struct BooksByTags<'a> {
    param: &'a [String],
}
impl<'a> BooksByTags<'a> {
    pub const QUERY: &'static str = r"SELECT 
  book_id,
  title,
  name,
  isbn,
  tags
FROM books
LEFT JOIN authors ON books.author_id = authors.author_id
WHERE tags && $1::varchar[]";
    pub fn query_many(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<Vec<BooksByTagsRow>, postgres::Error> {
        let rows = client.query(Self::QUERY, &[&self.param])?;
        rows.into_iter()
            .map(|r| BooksByTagsRow::from_row(&r))
            .collect()
    }
}
impl<'a> BooksByTags<'a> {
    pub const fn builder() -> BooksByTagsBuilder<'a, ((),)> {
        BooksByTagsBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct BooksByTagsBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> BooksByTagsBuilder<'a, ((),)> {
    pub fn param(self, param: &'a [String]) -> BooksByTagsBuilder<'a, (&'a [String],)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        BooksByTagsBuilder {
            fields: (param,),
            _phantom,
        }
    }
}
impl<'a> BooksByTagsBuilder<'a, (&'a [String],)> {
    pub const fn build(self) -> BooksByTags<'a> {
        let (param,) = self.fields;
        BooksByTags { param }
    }
}
pub struct CreateAuthorRow {
    pub authors_author_id: i32,
    pub authors_name: String,
}
impl CreateAuthorRow {
    fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {
            authors_author_id: row.try_get(0)?,
            authors_name: row.try_get(1)?,
        })
    }
}
pub struct CreateAuthor<'a> {
    authors_name: &'a str,
}
impl<'a> CreateAuthor<'a> {
    pub const QUERY: &'static str = r"INSERT INTO authors (name) VALUES ($1)
RETURNING author_id, name";
    pub fn query_one(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<CreateAuthorRow, postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.authors_name])?;
        CreateAuthorRow::from_row(&row)
    }
    pub fn query_opt(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<Option<CreateAuthorRow>, postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.authors_name])?;
        match row {
            Some(row) => Ok(Some(CreateAuthorRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateAuthor<'a> {
    pub const fn builder() -> CreateAuthorBuilder<'a, ((),)> {
        CreateAuthorBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct CreateAuthorBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> CreateAuthorBuilder<'a, ((),)> {
    pub fn authors_name(self, authors_name: &'a str) -> CreateAuthorBuilder<'a, (&'a str,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        CreateAuthorBuilder {
            fields: (authors_name,),
            _phantom,
        }
    }
}
impl<'a> CreateAuthorBuilder<'a, (&'a str,)> {
    pub const fn build(self) -> CreateAuthor<'a> {
        let (authors_name,) = self.fields;
        CreateAuthor { authors_name }
    }
}
pub struct CreateBookRow {
    pub books_book_id: i32,
    pub books_author_id: i32,
    pub books_isbn: String,
    pub books_book_type: BookType,
    pub books_title: String,
    pub books_year: i32,
    pub books_available: std::time::SystemTime,
    pub books_tags: Vec<String>,
}
impl CreateBookRow {
    fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {
            books_book_id: row.try_get(0)?,
            books_author_id: row.try_get(1)?,
            books_isbn: row.try_get(2)?,
            books_book_type: row.try_get(3)?,
            books_title: row.try_get(4)?,
            books_year: row.try_get(5)?,
            books_available: row.try_get(6)?,
            books_tags: row.try_get(7)?,
        })
    }
}
pub struct CreateBook<'a> {
    books_author_id: i32,
    books_isbn: &'a str,
    books_book_type: BookType,
    books_title: &'a str,
    books_year: i32,
    books_available: &'a std::time::SystemTime,
    books_tags: &'a [String],
}
impl<'a> CreateBook<'a> {
    pub const QUERY: &'static str = r"INSERT INTO books (
    author_id,
    isbn,
    book_type,
    title,
    year,
    available,
    tags
) VALUES (
    $1,
    $2,
    $3,
    $4,
    $5,
    $6,
    $7
)
RETURNING book_id, author_id, isbn, book_type, title, year, available, tags";
    pub fn query_one(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<CreateBookRow, postgres::Error> {
        let row = client.query_one(
            Self::QUERY,
            &[
                &self.books_author_id,
                &self.books_isbn,
                &self.books_book_type,
                &self.books_title,
                &self.books_year,
                &self.books_available,
                &self.books_tags,
            ],
        )?;
        CreateBookRow::from_row(&row)
    }
    pub fn query_opt(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<Option<CreateBookRow>, postgres::Error> {
        let row = client.query_opt(
            Self::QUERY,
            &[
                &self.books_author_id,
                &self.books_isbn,
                &self.books_book_type,
                &self.books_title,
                &self.books_year,
                &self.books_available,
                &self.books_tags,
            ],
        )?;
        match row {
            Some(row) => Ok(Some(CreateBookRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> CreateBook<'a> {
    pub const fn builder() -> CreateBookBuilder<'a, ((), (), (), (), (), (), ())> {
        CreateBookBuilder {
            fields: ((), (), (), (), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct CreateBookBuilder<'a, Fields = ((), (), (), (), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, BooksIsbn, BooksBookType, BooksTitle, BooksYear, BooksAvailable, BooksTags>
    CreateBookBuilder<
        'a,
        (
            (),
            BooksIsbn,
            BooksBookType,
            BooksTitle,
            BooksYear,
            BooksAvailable,
            BooksTags,
        ),
    >
{
    pub fn books_author_id(
        self,
        books_author_id: i32,
    ) -> CreateBookBuilder<
        'a,
        (
            i32,
            BooksIsbn,
            BooksBookType,
            BooksTitle,
            BooksYear,
            BooksAvailable,
            BooksTags,
        ),
    > {
        let ((), books_isbn, books_book_type, books_title, books_year, books_available, books_tags) =
            self.fields;
        let _phantom = self._phantom;
        CreateBookBuilder {
            fields: (
                books_author_id,
                books_isbn,
                books_book_type,
                books_title,
                books_year,
                books_available,
                books_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, BooksAuthorId, BooksBookType, BooksTitle, BooksYear, BooksAvailable, BooksTags>
    CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            (),
            BooksBookType,
            BooksTitle,
            BooksYear,
            BooksAvailable,
            BooksTags,
        ),
    >
{
    pub fn books_isbn(
        self,
        books_isbn: &'a str,
    ) -> CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            &'a str,
            BooksBookType,
            BooksTitle,
            BooksYear,
            BooksAvailable,
            BooksTags,
        ),
    > {
        let (
            books_author_id,
            (),
            books_book_type,
            books_title,
            books_year,
            books_available,
            books_tags,
        ) = self.fields;
        let _phantom = self._phantom;
        CreateBookBuilder {
            fields: (
                books_author_id,
                books_isbn,
                books_book_type,
                books_title,
                books_year,
                books_available,
                books_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, BooksAuthorId, BooksIsbn, BooksTitle, BooksYear, BooksAvailable, BooksTags>
    CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            BooksIsbn,
            (),
            BooksTitle,
            BooksYear,
            BooksAvailable,
            BooksTags,
        ),
    >
{
    pub fn books_book_type(
        self,
        books_book_type: BookType,
    ) -> CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            BooksIsbn,
            BookType,
            BooksTitle,
            BooksYear,
            BooksAvailable,
            BooksTags,
        ),
    > {
        let (books_author_id, books_isbn, (), books_title, books_year, books_available, books_tags) =
            self.fields;
        let _phantom = self._phantom;
        CreateBookBuilder {
            fields: (
                books_author_id,
                books_isbn,
                books_book_type,
                books_title,
                books_year,
                books_available,
                books_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, BooksAuthorId, BooksIsbn, BooksBookType, BooksYear, BooksAvailable, BooksTags>
    CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            BooksIsbn,
            BooksBookType,
            (),
            BooksYear,
            BooksAvailable,
            BooksTags,
        ),
    >
{
    pub fn books_title(
        self,
        books_title: &'a str,
    ) -> CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            BooksIsbn,
            BooksBookType,
            &'a str,
            BooksYear,
            BooksAvailable,
            BooksTags,
        ),
    > {
        let (
            books_author_id,
            books_isbn,
            books_book_type,
            (),
            books_year,
            books_available,
            books_tags,
        ) = self.fields;
        let _phantom = self._phantom;
        CreateBookBuilder {
            fields: (
                books_author_id,
                books_isbn,
                books_book_type,
                books_title,
                books_year,
                books_available,
                books_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, BooksAuthorId, BooksIsbn, BooksBookType, BooksTitle, BooksAvailable, BooksTags>
    CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            BooksIsbn,
            BooksBookType,
            BooksTitle,
            (),
            BooksAvailable,
            BooksTags,
        ),
    >
{
    pub fn books_year(
        self,
        books_year: i32,
    ) -> CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            BooksIsbn,
            BooksBookType,
            BooksTitle,
            i32,
            BooksAvailable,
            BooksTags,
        ),
    > {
        let (
            books_author_id,
            books_isbn,
            books_book_type,
            books_title,
            (),
            books_available,
            books_tags,
        ) = self.fields;
        let _phantom = self._phantom;
        CreateBookBuilder {
            fields: (
                books_author_id,
                books_isbn,
                books_book_type,
                books_title,
                books_year,
                books_available,
                books_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, BooksAuthorId, BooksIsbn, BooksBookType, BooksTitle, BooksYear, BooksTags>
    CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            BooksIsbn,
            BooksBookType,
            BooksTitle,
            BooksYear,
            (),
            BooksTags,
        ),
    >
{
    pub fn books_available(
        self,
        books_available: &'a std::time::SystemTime,
    ) -> CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            BooksIsbn,
            BooksBookType,
            BooksTitle,
            BooksYear,
            &'a std::time::SystemTime,
            BooksTags,
        ),
    > {
        let (books_author_id, books_isbn, books_book_type, books_title, books_year, (), books_tags) =
            self.fields;
        let _phantom = self._phantom;
        CreateBookBuilder {
            fields: (
                books_author_id,
                books_isbn,
                books_book_type,
                books_title,
                books_year,
                books_available,
                books_tags,
            ),
            _phantom,
        }
    }
}
impl<'a, BooksAuthorId, BooksIsbn, BooksBookType, BooksTitle, BooksYear, BooksAvailable>
    CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            BooksIsbn,
            BooksBookType,
            BooksTitle,
            BooksYear,
            BooksAvailable,
            (),
        ),
    >
{
    pub fn books_tags(
        self,
        books_tags: &'a [String],
    ) -> CreateBookBuilder<
        'a,
        (
            BooksAuthorId,
            BooksIsbn,
            BooksBookType,
            BooksTitle,
            BooksYear,
            BooksAvailable,
            &'a [String],
        ),
    > {
        let (
            books_author_id,
            books_isbn,
            books_book_type,
            books_title,
            books_year,
            books_available,
            (),
        ) = self.fields;
        let _phantom = self._phantom;
        CreateBookBuilder {
            fields: (
                books_author_id,
                books_isbn,
                books_book_type,
                books_title,
                books_year,
                books_available,
                books_tags,
            ),
            _phantom,
        }
    }
}
impl<'a>
    CreateBookBuilder<
        'a,
        (
            i32,
            &'a str,
            BookType,
            &'a str,
            i32,
            &'a std::time::SystemTime,
            &'a [String],
        ),
    >
{
    pub const fn build(self) -> CreateBook<'a> {
        let (
            books_author_id,
            books_isbn,
            books_book_type,
            books_title,
            books_year,
            books_available,
            books_tags,
        ) = self.fields;
        CreateBook {
            books_author_id,
            books_isbn,
            books_book_type,
            books_title,
            books_year,
            books_available,
            books_tags,
        }
    }
}
pub struct UpdateBookRow {}
impl UpdateBookRow {
    fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {})
    }
}
pub struct UpdateBook<'a> {
    books_title: &'a str,
    books_tags: &'a [String],
    books_book_id: i32,
}
impl<'a> UpdateBook<'a> {
    pub const QUERY: &'static str = r"UPDATE books
SET title = $1, tags = $2
WHERE book_id = $3";
    pub fn execute(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<u64, postgres::Error> {
        client.execute(
            Self::QUERY,
            &[&self.books_title, &self.books_tags, &self.books_book_id],
        )
    }
}
impl<'a> UpdateBook<'a> {
    pub const fn builder() -> UpdateBookBuilder<'a, ((), (), ())> {
        UpdateBookBuilder {
            fields: ((), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct UpdateBookBuilder<'a, Fields = ((), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, BooksTags, BooksBookId> UpdateBookBuilder<'a, ((), BooksTags, BooksBookId)> {
    pub fn books_title(
        self,
        books_title: &'a str,
    ) -> UpdateBookBuilder<'a, (&'a str, BooksTags, BooksBookId)> {
        let ((), books_tags, books_book_id) = self.fields;
        let _phantom = self._phantom;
        UpdateBookBuilder {
            fields: (books_title, books_tags, books_book_id),
            _phantom,
        }
    }
}
impl<'a, BooksTitle, BooksBookId> UpdateBookBuilder<'a, (BooksTitle, (), BooksBookId)> {
    pub fn books_tags(
        self,
        books_tags: &'a [String],
    ) -> UpdateBookBuilder<'a, (BooksTitle, &'a [String], BooksBookId)> {
        let (books_title, (), books_book_id) = self.fields;
        let _phantom = self._phantom;
        UpdateBookBuilder {
            fields: (books_title, books_tags, books_book_id),
            _phantom,
        }
    }
}
impl<'a, BooksTitle, BooksTags> UpdateBookBuilder<'a, (BooksTitle, BooksTags, ())> {
    pub fn books_book_id(
        self,
        books_book_id: i32,
    ) -> UpdateBookBuilder<'a, (BooksTitle, BooksTags, i32)> {
        let (books_title, books_tags, ()) = self.fields;
        let _phantom = self._phantom;
        UpdateBookBuilder {
            fields: (books_title, books_tags, books_book_id),
            _phantom,
        }
    }
}
impl<'a> UpdateBookBuilder<'a, (&'a str, &'a [String], i32)> {
    pub const fn build(self) -> UpdateBook<'a> {
        let (books_title, books_tags, books_book_id) = self.fields;
        UpdateBook {
            books_title,
            books_tags,
            books_book_id,
        }
    }
}
pub struct UpdateBookIsbnRow {}
impl UpdateBookIsbnRow {
    fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {})
    }
}
pub struct UpdateBookIsbn<'a> {
    books_title: &'a str,
    books_tags: &'a [String],
    books_book_id: i32,
    books_isbn: &'a str,
}
impl<'a> UpdateBookIsbn<'a> {
    pub const QUERY: &'static str = r"UPDATE books
SET title = $1, tags = $2, isbn = $4
WHERE book_id = $3";
    pub fn execute(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<u64, postgres::Error> {
        client.execute(
            Self::QUERY,
            &[
                &self.books_title,
                &self.books_tags,
                &self.books_book_id,
                &self.books_isbn,
            ],
        )
    }
}
impl<'a> UpdateBookIsbn<'a> {
    pub const fn builder() -> UpdateBookIsbnBuilder<'a, ((), (), (), ())> {
        UpdateBookIsbnBuilder {
            fields: ((), (), (), ()),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct UpdateBookIsbnBuilder<'a, Fields = ((), (), (), ())> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a, BooksTags, BooksBookId, BooksIsbn>
    UpdateBookIsbnBuilder<'a, ((), BooksTags, BooksBookId, BooksIsbn)>
{
    pub fn books_title(
        self,
        books_title: &'a str,
    ) -> UpdateBookIsbnBuilder<'a, (&'a str, BooksTags, BooksBookId, BooksIsbn)> {
        let ((), books_tags, books_book_id, books_isbn) = self.fields;
        let _phantom = self._phantom;
        UpdateBookIsbnBuilder {
            fields: (books_title, books_tags, books_book_id, books_isbn),
            _phantom,
        }
    }
}
impl<'a, BooksTitle, BooksBookId, BooksIsbn>
    UpdateBookIsbnBuilder<'a, (BooksTitle, (), BooksBookId, BooksIsbn)>
{
    pub fn books_tags(
        self,
        books_tags: &'a [String],
    ) -> UpdateBookIsbnBuilder<'a, (BooksTitle, &'a [String], BooksBookId, BooksIsbn)> {
        let (books_title, (), books_book_id, books_isbn) = self.fields;
        let _phantom = self._phantom;
        UpdateBookIsbnBuilder {
            fields: (books_title, books_tags, books_book_id, books_isbn),
            _phantom,
        }
    }
}
impl<'a, BooksTitle, BooksTags, BooksIsbn>
    UpdateBookIsbnBuilder<'a, (BooksTitle, BooksTags, (), BooksIsbn)>
{
    pub fn books_book_id(
        self,
        books_book_id: i32,
    ) -> UpdateBookIsbnBuilder<'a, (BooksTitle, BooksTags, i32, BooksIsbn)> {
        let (books_title, books_tags, (), books_isbn) = self.fields;
        let _phantom = self._phantom;
        UpdateBookIsbnBuilder {
            fields: (books_title, books_tags, books_book_id, books_isbn),
            _phantom,
        }
    }
}
impl<'a, BooksTitle, BooksTags, BooksBookId>
    UpdateBookIsbnBuilder<'a, (BooksTitle, BooksTags, BooksBookId, ())>
{
    pub fn books_isbn(
        self,
        books_isbn: &'a str,
    ) -> UpdateBookIsbnBuilder<'a, (BooksTitle, BooksTags, BooksBookId, &'a str)> {
        let (books_title, books_tags, books_book_id, ()) = self.fields;
        let _phantom = self._phantom;
        UpdateBookIsbnBuilder {
            fields: (books_title, books_tags, books_book_id, books_isbn),
            _phantom,
        }
    }
}
impl<'a> UpdateBookIsbnBuilder<'a, (&'a str, &'a [String], i32, &'a str)> {
    pub const fn build(self) -> UpdateBookIsbn<'a> {
        let (books_title, books_tags, books_book_id, books_isbn) = self.fields;
        UpdateBookIsbn {
            books_title,
            books_tags,
            books_book_id,
            books_isbn,
        }
    }
}
pub struct SayHelloRow {
    pub say_hello: Option<String>,
}
impl SayHelloRow {
    fn from_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {
            say_hello: row.try_get(0)?,
        })
    }
}
pub struct SayHello<'a> {
    s: &'a str,
}
impl<'a> SayHello<'a> {
    pub const QUERY: &'static str = r"select say_hello from say_hello($1)";
    pub fn query_one(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<SayHelloRow, postgres::Error> {
        let row = client.query_one(Self::QUERY, &[&self.s])?;
        SayHelloRow::from_row(&row)
    }
    pub fn query_opt(
        &self,
        client: &mut impl postgres::GenericClient,
    ) -> Result<Option<SayHelloRow>, postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[&self.s])?;
        match row {
            Some(row) => Ok(Some(SayHelloRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
impl<'a> SayHello<'a> {
    pub const fn builder() -> SayHelloBuilder<'a, ((),)> {
        SayHelloBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct SayHelloBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> SayHelloBuilder<'a, ((),)> {
    pub fn s(self, s: &'a str) -> SayHelloBuilder<'a, (&'a str,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        SayHelloBuilder {
            fields: (s,),
            _phantom,
        }
    }
}
impl<'a> SayHelloBuilder<'a, (&'a str,)> {
    pub const fn build(self) -> SayHello<'a> {
        let (s,) = self.fields;
        SayHello { s }
    }
}
