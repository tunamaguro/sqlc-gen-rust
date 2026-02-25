/* name: GetAuthor :one */
SELECT * FROM authors
WHERE id = ? LIMIT 1;

/* name: ListAuthors :many */
SELECT * FROM authors
ORDER BY name;

/* name: CreateAuthor :execresult */
INSERT INTO authors (
  name, bio
) VALUES (
  ?, ? 
);

/* name: DeleteAuthor :exec */
DELETE FROM authors
WHERE id = ?;

/* name: GetAuthorsByIds :many */
SELECT * FROM authors
WHERE id IN (sqlc.slice("ids"));

/* name: GetAuthorsByIdsAndName :many */
SELECT * FROM authors
WHERE name = ? AND id IN (sqlc.slice("ids"));