-- name: ListAuthorsByIDs :many
SELECT id, name
FROM authors
WHERE id IN (sqlc.slice('ids'))
ORDER BY id;

-- name: ListAuthorsByTwoIDLists :many
SELECT id, name
FROM authors
WHERE id IN (sqlc.slice('ids'))
   OR id IN (sqlc.slice('backup_ids'))
ORDER BY id;

-- name: ListAuthorsByIDsMixed :many
SELECT id, name
FROM authors
WHERE id IN (sqlc.slice('ids'))
  AND id >= ?
  AND id NOT IN (sqlc.slice('skip_ids'))
  AND name <> ?
ORDER BY id;
