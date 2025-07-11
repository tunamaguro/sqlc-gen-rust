-- name: GetAuthor :one
SELECT * FROM authors
WHERE id = $1 LIMIT 1;

-- name: CreateAuthors :copyfrom
INSERT INTO authors (id,name, bio) VALUES ($1, $2, $3);
