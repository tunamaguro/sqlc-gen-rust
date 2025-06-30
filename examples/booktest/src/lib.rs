#[allow(warnings)]
mod queries;

#[cfg(test)]
mod tests {
    use super::queries::*;
    use std::time::SystemTime;
    use test_context::test_context;
    use test_utils::PgSyncTestContext;

    fn migrate_db(client: &mut impl postgres::GenericClient) {
        client.batch_execute(include_str!("../schema.sql")).unwrap();
    }

    /// rewrite from https://github.com/sqlc-dev/sqlc/blob/v1.29.0/examples/booktest/postgresql/db_test.go
    #[test_context(PgSyncTestContext)]
    #[test]
    fn test_books(ctx: &mut PgSyncTestContext) {
        let client = &mut ctx.client;
        migrate_db(client);

        // create an author
        let create_author = CreateAuthor::builder()
            .authors_name("Unknown Master")
            .build();

        let a = create_author
            .query_one(client)
            .expect("Failed to create author");

        // create transaction
        let mut tx = client.transaction().expect("Failed to begin transaction");

        // save first book
        let now = SystemTime::now();
        let create_book1 = CreateBook::builder()
            .books_author_id(a.authors_author_id)
            .books_isbn("1")
            .books_title("my book title")
            .books_book_type(BookType::Fiction)
            .books_year(2016)
            .books_available(&now)
            .books_tags(&[])
            .build();

        let _b0 = create_book1
            .query_one(&mut tx)
            .expect("Failed to create first book");

        // save second book
        let tags2 = vec!["cool".to_string(), "unique".to_string()];
        let create_book2 = CreateBook::builder()
            .books_author_id(a.authors_author_id)
            .books_isbn("2")
            .books_title("the second book")
            .books_book_type(BookType::Fiction)
            .books_year(2016)
            .books_available(&now)
            .books_tags(&tags2)
            .build();

        let b1 = create_book2
            .query_one(&mut tx)
            .expect("Failed to create second book");

        // update the title and tags
        let update_tags = vec!["cool".to_string(), "disastor".to_string()];
        let update_book = UpdateBook::builder()
            .books_book_id(b1.books_book_id)
            .books_title("changed second title")
            .books_tags(&update_tags)
            .build();

        update_book.execute(&mut tx).expect("Failed to update book");

        // save third book
        let tags3 = vec!["cool".to_string()];
        let create_book3 = CreateBook::builder()
            .books_author_id(a.authors_author_id)
            .books_isbn("3")
            .books_title("the third book")
            .books_book_type(BookType::Fiction)
            .books_year(2001)
            .books_available(&now)
            .books_tags(&tags3)
            .build();

        let _b2 = create_book3
            .query_one(&mut tx)
            .expect("Failed to create third book");

        // save fourth book
        let tags4 = vec!["other".to_string()];
        let create_book4 = CreateBook::builder()
            .books_author_id(a.authors_author_id)
            .books_isbn("4")
            .books_title("4th place finisher")
            .books_book_type(BookType::Nonfiction)
            .books_year(2011)
            .books_available(&now)
            .books_tags(&tags4)
            .build();

        let b3 = create_book4
            .query_one(&mut tx)
            .expect("Failed to create fourth book");

        // tx commit
        tx.commit().expect("Failed to commit transaction");

        // upsert, changing ISBN and title
        let update_tags_isbn = vec!["someother".to_string()];
        let update_isbn = UpdateBookIsbn::builder()
            .books_book_id(b3.books_book_id)
            .books_isbn("NEW ISBN")
            .books_title("never ever gonna finish, a quatrain")
            .books_tags(&update_tags_isbn)
            .build();

        update_isbn
            .execute(client)
            .expect("Failed to update book ISBN");

        // retrieve first book
        let books_query = BooksByTitleYear::builder()
            .books_title("my book title")
            .books_year(2016)
            .build();

        let books0 = books_query
            .query_many(client)
            .expect("Failed to retrieve books by title and year");

        for book in &books0 {
            println!(
                "Book {} ({:?}): {} available: {:?}",
                book.books_book_id, book.books_book_type, book.books_title, book.books_available
            );

            let get_author = GetAuthor::builder()
                .authors_author_id(book.books_author_id)
                .build();

            let author = get_author.query_one(client).expect("Failed to get author");

            println!(
                "Book {} author: {}",
                book.books_book_id, author.authors_name
            );
        }

        // find a book with either "cool" or "other" tag
        println!("---------\nTag search results:");
        let search_tags = vec![
            "cool".to_string(),
            "other".to_string(),
            "someother".to_string(),
        ];
        let books_by_tags = BooksByTags::builder().param(&search_tags).build();

        let res = books_by_tags
            .query_many(client)
            .expect("Failed to search books by tags");

        for ab in &res {
            println!(
                "Book {}: '{}', Author: '{}', ISBN: '{}' Tags: '{:?}'",
                ab.books_book_id,
                ab.books_title,
                ab.authors_name.as_ref().unwrap_or(&"N/A".to_string()),
                ab.books_isbn,
                ab.books_tags
            );
        }

        // call function
        let say_hello = SayHello::builder().s("world").build();

        let result = say_hello
            .query_one(client)
            .expect("Failed to call say_hello function");

        let str_value = result.say_hello.expect("say_hello returned None");

        assert_eq!(
            str_value, "hello world",
            "expected function result to be \"hello world\". actual: {}",
            str_value
        );

        // get book 4 and delete
        let get_book = GetBook::builder().books_book_id(b3.books_book_id).build();

        let b5 = get_book.query_one(client).expect("Failed to get book");

        let delete_book = DeleteBook::builder()
            .books_book_id(b5.books_book_id)
            .build();

        delete_book.execute(client).expect("Failed to delete book");
    }
}
