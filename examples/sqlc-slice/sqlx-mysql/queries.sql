-- name: ListAuthorsByIDs :many
SELECT id, name
FROM authors
WHERE id IN (sqlc.slice('ids'))
ORDER BY id;
