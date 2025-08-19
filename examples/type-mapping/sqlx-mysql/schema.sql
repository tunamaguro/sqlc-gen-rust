CREATE TABLE mapping (
    bool_val BOOLEAN NOT NULL,
    tinyint_val TINYINT NOT NULL,
    smallint_val SMALLINT NOT NULL,
    int_val INT NOT NULL,
    int_nullable_val INT,
    bigint_val BIGINT NOT NULL,
    float_val FLOAT NOT NULL,
    double_val DOUBLE NOT NULL,
    text_val TEXT NOT NULL,
    blob_val BLOB NOT NULL,
    datetime_val DATETIME NOT NULL,
    date_val DATE NOT NULL,
    time_val TIME NOT NULL,
    json_val JSON NOT NULL,
    id BIGINT AUTO_INCREMENT PRIMARY KEY
);
