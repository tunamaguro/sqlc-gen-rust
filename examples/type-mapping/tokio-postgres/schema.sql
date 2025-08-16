CREATE EXTENSION hstore;

CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');

CREATE TYPE complex AS (
    r       double precision,
    i       double precision
);

CREATE TABLE mapping (
    bool_val BOOL NOT NULL,
    bool_array_val BOOL[] NOT NULL,
    char_val "char" NOT NULL,
    smallint_val SMALLINT NOT NULL,
    int_val INT NOT NULL,
    int_nullable_val INT,
    oid_val OID NOT NULL,
    bigint_val BIGINT NOT NULL,
    real_val REAL NOT NULL,
    double_val DOUBLE PRECISION NOT NULL,
    text_val TEXT NOT NULL,
    text_nullable_val TEXT,
    bytea_val BYTEA NOT NULL,
    hstore_val HSTORE NOT NULL,
    timestamp_val TIMESTAMP NOT NULL,
    timestamptz_val TIMESTAMPTZ NOT NULL,
    date_val DATE NOT NULL,
    time_val TIME NOT NULL,
    inet_val INET NOT NULL,
    json_val JSON NOT NULL,
    jsonb_val JSONB NOT NULL,
    uuid_val UUID NOT NULL,
    enum_val mood NOT NULL,
    composite_val complex NOT NULL,

    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY
);