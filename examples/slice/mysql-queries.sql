/* name: CreateAuthor :execresult */
INSERT INTO authors (
  name, bio
) VALUES (
  ?, ?
);

/* name: GetAuthorsByIds :many */
SELECT * FROM authors
WHERE id IN (sqlc.slice("ids"));

/* name: GetAuthorsByIdsAndName :many */
SELECT * FROM authors
WHERE name = ? AND id IN (sqlc.slice("ids"));
