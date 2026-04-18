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
    __query: String
}
```

この`__query`はビルド時に展開する。このようにする理由は`sqlx::QueryAs`がクエリ文字列へのライフタイムに制約されているため、`GetAuthorsByIds::query_as`を宣言できないためである。
`query_as`を返せるようにしないとユーザは`Stream`への対応が行えないため、これは必須である

おそらく、クエリ展開コードは次のようになる

```rust
pub struct GetAuthorsByIds<'a>{
    ids: &'a [i64],
    __query: String
}

impl <'a> GetAuthorsByIds<'a>{
    pub const QUERY: &'static str = r"SELECT * FROM authors
WHERE id IN (/*SLICE:ids*/?)";
}


impl <'a> GetAuthorsBulder<'a,(&'a [i64])>{
    pub fn build(self) -> GetAuthorsByIds<'a>{
        let (ids,) = self.fields;
        let query = match ids.len(){
           0 => {
               GetAuthorsByIds::Query.replacen("/*SLICE:ids*/?","NULL",1)
           },
           1 => {
               GetAuthorsByIds::Query.replacen("/*SLICE:ids*/?","?",1)
           },
           _ => {
               let tmp = core::iter::once("?")
                   .chain(core::iter::repeat_n(",?", arr.len() - 1))
                   .collect::<String>();
               GetAuthorsByIds::Query.replacen("/*SLICE:ids*/?",tmp,1)
           }
        }
   
        // other replacements...

        GetAuthorsByIds{
            ids,
            __query : query
        }
    }
}
```