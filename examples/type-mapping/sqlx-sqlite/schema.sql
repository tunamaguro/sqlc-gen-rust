CREATE TABLE mapping (
    bool_val BOOLEAN NOT NULL,
    int_val INTEGER NOT NULL,
    int_nullable_val INTEGER,
    real_val REAL NOT NULL,
    text_val TEXT NOT NULL,
    blob_val BLOB NOT NULL,
    datetime_val DATETIME NOT NULL,
    date_val DATE NOT NULL,
    time_val TIME NOT NULL,
    json_val TEXT NOT NULL,
    id INTEGER PRIMARY KEY AUTOINCREMENT
);
