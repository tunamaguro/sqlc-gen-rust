CREATE TABLE mapping (
  aff_integer_val         INTEGER    NOT NULL,
  aff_real_val            REAL       NOT NULL,
  aff_text_val            TEXT       NOT NULL,
  aff_blob_val            BLOB       NOT NULL,

  int_val                 INT        NOT NULL,
  integer_val             INTEGER    NOT NULL,
  tinyint_val             TINYINT    NOT NULL,
  smallint_val            SMALLINT   NOT NULL,
  mediumint_val           MEDIUMINT  NOT NULL,
  bigint_val              BIGINT     NOT NULL,
  unsigned_big_int_val    UNSIGNED BIG INT NOT NULL,
  int2_val                INT2       NOT NULL,
  int8_val                INT8       NOT NULL,

  character20_val         CHARACTER(20) NOT NULL,
  varchar255_val          VARCHAR(255) NOT NULL,
  varying_char255_val     VARYING CHARACTER(255) NOT NULL,
  nchar55_val             NCHAR(55)   NOT NULL,
  native_char70_val       NATIVE CHARACTER(70) NOT NULL,
  nvarchar100_val         NVARCHAR(100) NOT NULL,
  text_val                TEXT        NOT NULL,
  clob_val                CLOB        NOT NULL,

  real_val                REAL        NOT NULL,
  double_val              DOUBLE      NOT NULL,
  double_precision_val    DOUBLE PRECISION NOT NULL,
  float_val               FLOAT       NOT NULL,

  numeric_val             NUMERIC     NOT NULL,
  decimal10_5_val         DECIMAL(10,5) NOT NULL,
  boolean_val             BOOLEAN     NOT NULL,
  datetime_val            DATETIME    NOT NULL,
  date_val                DATE        NOT NULL,
  time_val                TIME        NOT NULL,

  id_val                  INTEGER PRIMARY KEY AUTOINCREMENT
);
