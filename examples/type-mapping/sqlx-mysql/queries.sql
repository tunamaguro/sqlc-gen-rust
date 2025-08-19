/* name: GetMapping :one */
SELECT
    bool_val,
    tinyint_val,
    smallint_val,
    int_val,
    int_nullable_val,
    bigint_val,
    float_val,
    double_val,
    text_val,
    blob_val,
    timestamp_val,
    datetime_val,
    date_val,
    time_val,
    json_val
FROM mapping;

/* name: InsertMapping :exec */
INSERT INTO mapping (
    bool_val,
    tinyint_val,
    smallint_val,
    int_val,
    int_nullable_val,
    bigint_val,
    float_val,
    double_val,
    text_val,
    blob_val,
    timestamp_val,
    datetime_val,
    date_val,
    time_val,
    json_val
) VALUES (
    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
);
