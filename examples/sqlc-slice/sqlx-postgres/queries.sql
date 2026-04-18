-- name: ListAuthorsByIDs :many
SELECT id, name
FROM authors
WHERE id = ANY(sqlc.slice('ids')::bigint[])
ORDER BY id;

-- name: ListAuthorsByTwoIDLists :many
SELECT id, name
FROM authors
WHERE id = ANY(sqlc.slice('ids')::bigint[])
   OR id = ANY(sqlc.slice('backup_ids')::bigint[])
ORDER BY id;

-- name: ListAuthorsByIDsMixed :many
SELECT id, name
FROM authors
WHERE id = ANY(sqlc.slice('ids')::bigint[])
  AND id >= sqlc.arg('min_id')
  AND NOT (id = ANY(sqlc.slice('skip_ids')::bigint[]))
  AND name <> sqlc.arg('excluded_name')
ORDER BY id;
