pub struct CountPilotsRow {
    pub count: i64,
}
impl CountPilotsRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            count: row.try_get(0)?,
        })
    }
}
pub struct CountPilots;
impl CountPilots {
    pub const QUERY: &'static str = r"SELECT COUNT(*) FROM pilots";
    pub async fn query_one(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<CountPilotsRow, deadpool_postgres::tokio_postgres::Error> {
        let row = client.query_one(Self::QUERY, &[]).await?;
        CountPilotsRow::from_row(&row)
    }
    pub async fn query_opt(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Option<CountPilotsRow>, deadpool_postgres::tokio_postgres::Error> {
        let row = client.query_opt(Self::QUERY, &[]).await?;
        match row {
            Some(row) => Ok(Some(CountPilotsRow::from_row(&row)?)),
            None => Ok(None),
        }
    }
}
pub struct ListPilotsRow {
    pub id: i32,
    pub name: String,
}
impl ListPilotsRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {
            id: row.try_get(0)?,
            name: row.try_get(1)?,
        })
    }
}
pub struct ListPilots;
impl ListPilots {
    pub const QUERY: &'static str = r"SELECT id, name FROM pilots LIMIT 5";
    pub async fn query_many(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<Vec<ListPilotsRow>, deadpool_postgres::tokio_postgres::Error> {
        let rows = client.query(Self::QUERY, &[]).await?;
        rows.into_iter()
            .map(|r| ListPilotsRow::from_row(&r))
            .collect()
    }
}
pub struct DeletePilotRow {}
impl DeletePilotRow {
    fn from_row(
        row: &deadpool_postgres::tokio_postgres::Row,
    ) -> Result<Self, deadpool_postgres::tokio_postgres::Error> {
        Ok(Self {})
    }
}
pub struct DeletePilot {
    id: i32,
}
impl DeletePilot {
    pub const QUERY: &'static str = r"DELETE FROM pilots WHERE id = $1";
    pub async fn execute(
        &self,
        client: &impl deadpool_postgres::GenericClient,
    ) -> Result<u64, deadpool_postgres::tokio_postgres::Error> {
        client.execute(Self::QUERY, &[&self.id]).await
    }
}
impl DeletePilot {
    pub const fn builder() -> DeletePilotBuilder<'static, ((),)> {
        DeletePilotBuilder {
            fields: ((),),
            _phantom: std::marker::PhantomData,
        }
    }
}
pub struct DeletePilotBuilder<'a, Fields = ((),)> {
    fields: Fields,
    _phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> DeletePilotBuilder<'a, ((),)> {
    pub fn id(self, id: i32) -> DeletePilotBuilder<'a, (i32,)> {
        let ((),) = self.fields;
        let _phantom = self._phantom;
        DeletePilotBuilder {
            fields: (id,),
            _phantom,
        }
    }
}
impl<'a> DeletePilotBuilder<'a, (i32,)> {
    pub const fn build(self) -> DeletePilot {
        let (id,) = self.fields;
        DeletePilot { id }
    }
}
