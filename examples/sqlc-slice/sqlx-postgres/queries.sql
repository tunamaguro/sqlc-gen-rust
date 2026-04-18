-- name: ListAuthorsByIDs :many
SELECT id, name
FROM authors
WHERE id = ANY(sqlc.slice('ids')::bigint[])
ORDER BY id;
