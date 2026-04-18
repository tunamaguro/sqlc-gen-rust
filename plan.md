# sqlc.slice 対応方針


## sqlx

### postgres

`&[T]`にマッピングする


### mysql/sqlite

実行時に`/*SLICE:name*/?`のようなマーカーを`(?,?,?...)`に展開し、引数は`&[T]`で受けてバインドする。
ただしこの時、展開したクエリを保持させる。具体的には次のイメージ

```sql
/* name: GetAuthorsByIds :many */
SELECT * FROM authors
WHERE id IN (sqlc.slice("ids"));
```

```rust
struct GetAuthorsByIds<'a>{
    ids: &'a [i64],
    __query: std::borrow::Cow<'static,str>
}
```

この`__query`はビルド時に展開する。このようにする理由は`sqlx::QueryAs`がクエリ文字列へのライフタイムに制約されているため、`GetAuthorsByIds::query_as`を宣言できないためである。
`query_as`を返せるようにしないとユーザは`Stream`への対応が行えないため、これは必須である
